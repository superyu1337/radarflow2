use std::sync::Arc;

use csflow::{CheatCtx, Connector, memflow::Process, traits::{MemoryClass, BaseEntity}, enums::PlayerType, structs::{CBaseEntity, CPlayerController}};
use tokio::{sync::RwLock, time::{Duration, Instant}};

use crate::{comms::{RadarData, EntityData, BombData, PlayerData}, dma::cache::CacheBuilder};

use self::cache::Cache;

mod cache;

const SECOND_AS_NANO: u64 = 1000*1000*1000;
static ONCE: std::sync::Once = std::sync::Once::new();

pub async fn run(connector: Connector, pcileech_device: String, poll_rate: u16, data_lock: Arc<RwLock<RadarData>>) -> anyhow::Result<()> {
    let mut ctx = CheatCtx::setup(connector, pcileech_device)?;

    println!("---------------------------------------------------");
    println!("Found cs2.exe at {:X}", ctx.process.info().address);
    println!("Found engine module at cs2.exe+{:X}", ctx.engine_module.base);
    println!("Found client module at cs2.exe+{:X}", ctx.client_module.base);
    println!("---------------------------------------------------");

    // Avoid printing warnings and other stuff before the initial prints are complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // For poll rate timing
    let should_time = poll_rate != 0;

    let target_interval = Duration::from_nanos(SECOND_AS_NANO / poll_rate as u64);
    let mut last_iteration_time = Instant::now();
    let mut missmatch_count = 0;

    let mut cache = Cache::new_invalid();

    loop {
        if ctx.process.state().is_dead() {
            break;
        }

        if !cache.is_valid() {
            let mut cached_entities = Vec::new();

            let globals = ctx.get_globals()?;
            let highest_index = ctx.highest_entity_index()?;
            let map_name = ctx.map_name(globals)?;
            let entity_list = ctx.get_entity_list()?;

            let local = ctx.get_local()?;
            
            if local.get_pawn(&mut ctx, entity_list)?.is_some() {
                cached_entities.push(cache::CachedEntity::Player {
                    ptr: local.ptr(),
                    player_type: PlayerType::Local 
                });

                for idx in 1..=highest_index {
                    if let Some(entity) = CBaseEntity::from_index(&mut ctx, entity_list, idx)? {

                        let class_name = entity.class_name(&mut ctx)?;

                        match class_name.as_str() {
                            "weapon_c4" => {
                                cached_entities.push(cache::CachedEntity::Bomb {
                                    ptr: entity.ptr()
                                })
                            },
                            "cs_player_controller" => {
                                let controller = entity.to_player_controller();

                                let player_type = {
                                    match controller.get_player_type(&mut ctx, &local)? {
                                        Some(t) => {
                                            if t == PlayerType::Spectator { continue } else { t }
                                        },
                                        None => { continue },
                                    }
                                };

                                cached_entities.push(cache::CachedEntity::Player {
                                    ptr: entity.ptr(),
                                    player_type,
                                })
                            }
                            _ => {}
                        }
                    }
                }
            }

            cache = CacheBuilder::new()
                .entity_cache(cached_entities)
                .entity_list(entity_list)
                .map_name(map_name)
                .build()?;

            log::debug!("Rebuilt cache.");
        }

        if ctx.network_is_ingame()? {
            let mut radar_data = Vec::with_capacity(64);

            if ctx.is_bomb_planted()? {
                let bomb = ctx.get_plantedc4()?;
                let bomb_pos = bomb.pos(&mut ctx)?;
                radar_data.push(
                    EntityData::Bomb(BombData::new(
                        bomb_pos,
                        true
                    ))
                );
            }

            for cached_data in cache.entity_cache() {
                match cached_data {
                    cache::CachedEntity::Bomb { ptr } => {
                        if ctx.is_bomb_dropped()?  {
                            let bomb_entity = CBaseEntity::new(ptr);
                            let pos = bomb_entity.pos(&mut ctx)?;
    
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
                    cache::CachedEntity::Player { ptr, player_type } => {
                        let controller = CPlayerController::new(ptr);
                        if let Some(pawn) = controller.get_pawn(&mut ctx, cache.entity_list())? {
                            if pawn.is_alive(&mut ctx)? {
                                let pos = pawn.pos(&mut ctx)?;
                                let yaw = pawn.angles(&mut ctx)?.y;
                                let has_bomb = pawn.has_c4(&mut ctx, cache.entity_list())?;
                    
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
            if cache.map_name() == "<empty>" || cache.map_name().is_empty() {
                *data = RadarData::empty()
            } else {
                *data = RadarData::new(true, cache.map_name(), radar_data)
            }
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
            log::trace!("elapsed: {}ns", elapsed.as_nanos());
            log::trace!("target: {}ns", target_interval.as_nanos());
            log::trace!("missmatch count: {}", missmatch_count);

            last_iteration_time = Instant::now();
        }
    }

    Ok(())
}
