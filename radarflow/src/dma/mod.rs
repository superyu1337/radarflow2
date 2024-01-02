use std::sync::Arc;

use csflow::{CheatCtx, Connector, memflow::Process, traits::{MemoryClass, BaseEntity}, enums::PlayerType, structs::{CBaseEntity, CPlayerController}};
use tokio::{sync::RwLock, time::Duration};

use crate::{comms::{RadarData, EntityData, BombData, PlayerData}, dma::cache::CacheBuilder};

use self::cache::Cache;

mod cache;

const SECOND_AS_NANO: u64 = 1000*1000*1000;

pub async fn run(connector: Connector, pcileech_device: String, data_lock: Arc<RwLock<RadarData>>) -> anyhow::Result<()> {
    let mut ctx = CheatCtx::setup(connector, pcileech_device)?;

    let mut cache = Cache::new_invalid();

    let mut last_tickcount = -1;
    let mut last_round = -1;
    let mut last_gamephase = -1;
    let mut last_bomb_dropped = false;

    // Duration for a single tick on 128 ticks. I'm assuming 128 ticks because I don't fucking know how to read the current tickrate off cs2 memory lol
    let target_interval = Duration::from_nanos(SECOND_AS_NANO / 128);
    
    loop {
        let start_stamp = tokio::time::Instant::now();

        if ctx.process.state().is_dead() {
            break;
        }

        if !cache.is_valid() {
            let mut cached_entities = Vec::new();

            let globals = ctx.get_globals()?;
            let highest_index = ctx.highest_entity_index()?;
            let entity_list = ctx.get_entity_list()?;
            let gamerules = ctx.get_gamerules()?;

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
                .globals(globals)
                .gamerules(gamerules)
                .build()?;

            log::info!("Rebuilt cache.");
        }

        if ctx.network_is_ingame()? {
            // Check if mapname is "<empty>"
            // That means we are not in-game, so we can just write empty radar data and run the next loop.
            let map_name = cache.globals().map_name(&mut ctx)?;

            if map_name == "<empty>" {
                last_round = -1;
                last_gamephase = -1;

                let mut data = data_lock.write().await;
                *data = RadarData::empty();
                continue;
            } else if map_name.is_empty() { // Check if mapname is empty, this usually means a bad globals pointer -> rebuild our cache
                cache.invalidate();
                log::info!("Invalidated cache! Reason: invalid globals pointer");
                continue;
            }

            let cur_round = cache.gamerules().total_rounds_played(&mut ctx)?;

            // New round started, invalidate cache and run next loop
            if cur_round != last_round {
                last_round = cur_round;
                cache.invalidate();
                log::info!("Invalidated cache! Reason: new round");
                continue;
            }

            let cur_gamephase = cache.gamerules().game_phase(&mut ctx)?;

            // New game phase, invalidate cache and run next loop
            if cur_gamephase != last_gamephase {
                last_gamephase = cur_gamephase;
                cache.invalidate();
                log::info!("Invalidated cache! Reason: new gamephase");
                continue;
            }

            let cur_bomb_dropped = cache.gamerules().bomb_dropped(&mut ctx)?;

            if cur_bomb_dropped != last_bomb_dropped {
                last_bomb_dropped = cur_bomb_dropped;
                cache.invalidate();
                log::info!("Invalidated cache! Reason: bomb drop status changed");
                continue;
            }

            let cur_tickcount = cache.globals().tick_count(&mut ctx)?;

            // New tick, now we want to fetch our data
            if cur_tickcount != last_tickcount {
                // We don't expect more than 16 entries in our radar data.
                let mut radar_data = Vec::with_capacity(16);

                if cache.gamerules().bomb_planted(&mut ctx)? {
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
                            if cache.gamerules().bomb_dropped(&mut ctx)? {
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
                *data = RadarData::new(
                    true,
                    map_name,
                    radar_data
                );

                last_tickcount = cur_tickcount;
            }
        }

        // Elapsed time since we started our loop
        let elapsed = start_stamp.elapsed();
        
        let remaining = match target_interval.checked_sub(elapsed) {
            // This gives us the remaining time we can sleep in our loop
            Some(t) => t,
            // No time left, start next loop.
            None => continue
        };

        // On linux we may use tokio_timerfd for a more finely grained sleep function
        #[cfg(target_os = "linux")]
        tokio_timerfd::sleep(remaining).await?;

        // On non linux build targets we need to use the regular sleep function, this one is only accurate to millisecond precision 
        #[cfg(not(target_os = "linux"))]
        tokio::time::sleep(remaining).await;

        log::debug!("poll rate: {:.2}Hz", SECOND_AS_NANO as f64 / start_stamp.elapsed().as_nanos() as f64);
        log::debug!("elapsed: {}ns", elapsed.as_nanos());
        log::debug!("target: {}ns", target_interval.as_nanos());
    }

    Ok(())
}
