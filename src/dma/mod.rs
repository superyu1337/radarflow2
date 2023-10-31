use ::std::sync::Arc;

use memflow::prelude::v1::*;
use tokio::{sync::RwLock, time::{Duration, Instant}};

use crate::{structs::{Connector, communication::{RadarData, PlayerType, EntityData, PlayerData}}, sdk::{self, cs2dumper, structs::{CPlayerPawn, CCSPlayerController}}};

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

    loop {
        if ctx.process.state().is_dead() {
            println!("is dead");
            break;
        }

        if sdk::is_ingame(&mut ctx)? {
            let globals = sdk::get_globals(&mut ctx)?;
            let entity_list = sdk::get_entity_list(&mut ctx)?;
            let map_name = sdk::map_name(globals, &mut ctx)?;

            let local = sdk::get_local(&mut ctx)?;

            let local_pawn_ptr: u32 = ctx.process.read(local.ptr() + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?; 
            
            if let Some(local_pawn) = CPlayerPawn::from_uhandle(local_pawn_ptr, entity_list, &mut ctx) {
                let local_yaw = local_pawn.angles(&mut ctx)?.y;
                let local_pos = local_pawn.pos(&mut ctx)?;
                let mut player_data = Vec::with_capacity(64);
    
                if local_pawn.is_alive(&mut ctx)? {
                    player_data.push(
                        EntityData::Player(
                            PlayerData::new(
                                local_pos, 
                                local_yaw,
                                PlayerType::Local,
                                false
                            )
                        )
                    );
                }

                let max_clients = sdk::max_clients(globals, &mut ctx)?;

                for idx in 1..max_clients {
                    let list_entry = ctx.process.read_addr64(entity_list + ((8 * (idx & 0x7FFF)) >> 9) + 16)?;
                    if list_entry.is_null() && !list_entry.is_valid() {
                        continue;
                    }
    
                    let player_ptr = ctx.process.read_addr64(list_entry + 120 * (idx & 0x1FF))?;
                    if player_ptr.is_null() && !player_ptr.is_valid() {
                        continue;
                    }
    
                    let pawn_uhandle = ctx.process.read(player_ptr + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?;
    
                    if let (Some(pawn), player) = (CPlayerPawn::from_uhandle(pawn_uhandle, entity_list, &mut ctx), CCSPlayerController::new(player_ptr)) {
                        if player.entity_identity(&mut ctx)?.designer_name(&mut ctx)? == "cs_player_controller" && pawn.is_alive(&mut ctx)? {
                            let pos = pawn.pos(&mut ctx)?;
                            let angles = pawn.angles(&mut ctx)?;
                
                            let player_type = {
                                match player.get_player_type(&mut ctx, &local)? {
                                    Some(t) => {
                                        if t == PlayerType::Spectator { continue } else { t }
                                    },
                                    None => { continue },
                                }
                            };
            
                            player_data.push(
                                EntityData::Player(
                                    PlayerData::new(
                                        pos, 
                                        angles.y,
                                        player_type,
                                        false
                                    )
                                )
                            );
                        }
                    }
                }
    
                let mut data = data_lock.write().await;
                *data = RadarData::new(true, map_name, player_data, local_yaw)
            }




            //let local_pawn = sdk::get_local_pawn(&mut ctx)?;
            //let local_pawn = CPlayerPawn::new(local_cs_player_pawn);





            /*

            let mut next_ent = {
                let mut iter_ent = local.to_base();
                while iter_ent.entity_identity(&mut ctx)?.prev_by_class(&mut ctx).is_ok() {
                    iter_ent = iter_ent.entity_identity(&mut ctx)?.prev_by_class(&mut ctx)?;
                }
    
                iter_ent
            };

            let mut count = 0;
            let mut pawn_count = 0;

            println!("prev by class ok? {}", next_ent.entity_identity(&mut ctx)?.prev_by_class(&mut ctx).is_ok());

            while next_ent.entity_identity(&mut ctx)?.next_by_class(&mut ctx).is_ok() {
                count += 1;
                let pawn = next_ent.to_controller().pawn(entity_list, &mut ctx)?;

                if let Some(p) = pawn {
                    pawn_count += 1;
                    if !p.is_alive(&mut ctx)? {
                        next_ent = next_ent.entity_identity(&mut ctx).unwrap().next_by_class(&mut ctx).unwrap();
                        continue
                    }
    
                    let pos = p.pos(&mut ctx)?;
                    let angles = p.angles(&mut ctx)?;
        
                    let player_type = {
                        match next_ent.to_controller().get_player_type(&mut ctx, &local)? {
                            Some(t) => {
                                if t == PlayerType::Spectator {
                                    next_ent = next_ent.entity_identity(&mut ctx).unwrap().next_by_class(&mut ctx).unwrap();
                                    continue
                                } else { t }
                            },
                            None => {
                                next_ent = next_ent.entity_identity(&mut ctx).unwrap().next_by_class(&mut ctx).unwrap();
                                continue 
                            },
                        }
                    };
    
                    player_data.push(
                        EntityData::Player(
                            PlayerData::new(
                                pos, 
                                angles.y,
                                player_type,
                                false
                            )
                        )
                    );
                }
                //let pawn = next_ent.to_controller().pawn2(entity_list, &mut ctx)?;

                next_ent = next_ent.entity_identity(&mut ctx)?.next_by_class(&mut ctx)?;
            }

            println!("next by class ok? {}", next_ent.entity_identity(&mut ctx)?.next_by_class(&mut ctx).is_ok());

            */

        } else {
            let mut data = data_lock.write().await;
            *data = RadarData::empty();
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
    
            log::trace!("polling at {:.2}Hz", SECOND_AS_NANO as f64 / last_iteration_time.elapsed().as_nanos() as f64);
            log::trace!("elapsed: {}", elapsed.as_nanos());
            log::trace!("target: {}", target_interval.as_nanos());
            log::trace!("missmatch count: {}", missmatch_count);
    
            last_iteration_time = Instant::now();
        }
    }

    println!("DMA loop exited for some reason");

    Ok(())
}
