use ::std::sync::Arc;

use memflow::prelude::v1::*;
use tokio::{sync::RwLock, time::{Duration, Instant}};

use crate::{structs::{Connector, communication::{RadarData, PlayerType, EntityData, PlayerData, BombData}}, sdk::{self, structs::{PlayerController, Cache, MemoryClass}}};

pub struct CheatCtx {
    pub process: IntoProcessInstanceArcBox<'static>,
    pub client_module: ModuleInfo,
    pub engine_module: ModuleInfo,
}

impl CheatCtx {
    pub fn setup(connector: Connector, pcileech_device: String) -> anyhow::Result<CheatCtx> {
        let inventory = Inventory::scan();

        let os = { 
            if connector == Connector::Pcileech {
                let args = Args::new()
                    .insert("device", &pcileech_device);

                let connector_args = ConnectorArgs::new(None, args, None);                

                inventory.builder()
                    .connector(&connector.to_string())
                    .args(connector_args)
                    .os("win32")
                    .build()?
            } else {
                inventory.builder()
                .connector(&connector.to_string())
                .os("win32")
                .build()?
            }
        };

        let mut process = os.into_process_by_name("cs2.exe")?;

        let client_module = process.module_by_name("client.dll")?;

        let engine_module = process.module_by_name("engine2.dll")?;

        let ctx = Self {
            process,
            client_module,
            engine_module,
        };

        Ok(ctx)
    }
}

const SECOND_AS_NANO: u64 = 1000*1000*1000;
static ONCE: std::sync::Once = std::sync::Once::new();

pub async fn run(connector: Connector, pcileech_device: String, poll_rate: u16, data_lock: Arc<RwLock<RadarData>>) -> anyhow::Result<()> {
    let mut ctx = CheatCtx::setup(connector, pcileech_device)?;

    // Avoid printing warnings and other stuff before the initial prints are complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // For poll rate timing
    let should_time = poll_rate != 0;

    let target_interval = Duration::from_nanos(SECOND_AS_NANO / poll_rate as u64);
    let mut last_iteration_time = Instant::now();
    let mut missmatch_count = 0;

    let mut cache = Cache::new();

    loop {
        if ctx.process.state().is_dead() {
            break;
        }

        if cache.is_outdated() {
            cache.clean();

            let globals = sdk::get_globals(&mut ctx)?;
            let highest_index = sdk::highest_entity_index(&mut ctx)?;
            let map_name = sdk::map_name(globals, &mut ctx)?;
            let entity_list = sdk::get_entity_list(&mut ctx)?;

            cache.common().update(
                map_name,
                entity_list
            );

            let local = sdk::get_local(&mut ctx)?;
            
            if local.pawn(&mut ctx, entity_list)?.is_some() {
                cache.push_data(sdk::structs::CachedEntityData::Player {
                    ptr: local.ptr(),
                    player_type: PlayerType::Local 
                });

                for idx in 1..highest_index {
                    if let Some(entity) = PlayerController::from_entity_list_v2(&mut ctx, entity_list, idx)? {

                        let class_name = entity.class_name(&mut ctx)?;

                        match class_name.as_str() {
                            "weapon_c4" => {
                                cache.push_data(sdk::structs::CachedEntityData::Bomb {
                                    ptr: entity.ptr()
                                })
                            },
                            "cs_player_controller" => {
                                let player_type = {
                                    match entity.get_player_type(&mut ctx, &local)? {
                                        Some(t) => {
                                            if t == PlayerType::Spectator { continue } else { t }
                                        },
                                        None => { continue },
                                    }
                                };

                                cache.push_data(sdk::structs::CachedEntityData::Player {
                                    ptr: entity.ptr(),
                                    player_type,
                                })
                            }
                            _ => {}
                        }
                    }
                }
            }

            log::debug!("Rebuilt cache.");

            cache.new_time();
        }

        if sdk::is_ingame(&mut ctx)? {
            let mut radar_data = Vec::with_capacity(64);

            if sdk::is_bomb_planted(&mut ctx)? {
                let bomb = sdk::get_plantedc4(&mut ctx)?;
                let bomb_pos = bomb.pos(&mut ctx)?;
                radar_data.push(
                    EntityData::Bomb(BombData::new(
                        bomb_pos,
                        true
                    ))
                );
            }

            for cached_data in cache.data() {
                match cached_data {
                    sdk::structs::CachedEntityData::Bomb { ptr } => {
                        if sdk::is_bomb_dropped(&mut ctx)?  {
                            let controller = PlayerController::new(ptr);
                            let pos = controller.pos(&mut ctx)?;
    
                            radar_data.push(
                                EntityData::Bomb(
                                    BombData::new(
                                        pos,
                                        false
                                    )
                                )
                            );
                        }
                    },
                    sdk::structs::CachedEntityData::Player { ptr, player_type } => {
                        let controller = PlayerController::new(ptr);
                        if let Some(pawn) = controller.pawn(&mut ctx, cache.common().entity_list())? {
                            if pawn.is_alive(&mut ctx)? {
                                let pos = pawn.pos(&mut ctx)?;
                                let yaw = pawn.angles(&mut ctx)?.y;
                                let has_bomb = pawn.has_c4(&mut ctx, cache.common().entity_list())?;
                    
                                radar_data.push(
                                    EntityData::Player(
                                        PlayerData::new(
                                            pos, 
                                            yaw,
                                            player_type,
                                            has_bomb
                                        )
                                    )
                                );
                            }
                        }
                    },
                }
            }

            let mut data = data_lock.write().await;
            *data = RadarData::new(true, cache.common().map_name(), radar_data)
        } else {
            let mut data = data_lock.write().await;
            *data = RadarData::empty()
        }

        if should_time {
            let elapsed = last_iteration_time.elapsed();
    
            let remaining = match target_interval.checked_sub(elapsed) {
                Some(t) => t,
                None => {
                    if missmatch_count >= 25 {
                        ONCE.call_once(|| {
                            log::warn!("Remaining time till target interval was negative more than 25 times");
                            log::warn!("You should decrease your poll rate.");
                            log::warn!("elapsed: {}ns", elapsed.as_nanos());
                            log::warn!("target: {}ns", target_interval.as_nanos());
                        });
                    } else {
                        missmatch_count += 1;
                    }
                    Duration::from_nanos(0)
                },
            };
    
            #[cfg(target_os = "linux")]
            tokio_timerfd::sleep(remaining).await?;
    
            #[cfg(not(target_os = "linux"))]
            tokio::time::sleep(remaining).await;
    
            log::info!("poll rate: {:.2}Hz", SECOND_AS_NANO as f64 / last_iteration_time.elapsed().as_nanos() as f64);
            log::trace!("elapsed: {}", elapsed.as_nanos());
            log::trace!("target: {}", target_interval.as_nanos());
            log::trace!("missmatch count: {}", missmatch_count);
    
            last_iteration_time = Instant::now();
        }
    }

    Ok(())
}
