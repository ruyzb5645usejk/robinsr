use lazy_static::lazy_static;
use prost::Message;
use tokio::sync::Mutex;

use crate::{
    net::{
        tools::{JsonData, Position},
        tools_res::{PropState, GAME_RESOURCES},
    },
    util,
};

use super::*;

#[derive(Message)]
pub struct Dummy {}

pub async fn on_get_cur_scene_info_cs_req(
    session: &mut PlayerSession,
    _body: &GetCurSceneInfoCsReq,
) -> Result<()> {
    let mut player = JsonData::load().await;

    let scene = load_scene(&mut player).await;

    let resp = GetCurSceneInfoScRsp {
        retcode: 0,
        scene: if let Ok(scene) = scene {
            Some(scene)
        } else {
            Some(SceneInfo {
                plane_id: player.scene.plane_id,
                floor_id: player.scene.floor_id,
                entry_id: player.scene.entry_id,
                game_mode_type: 1,
                ..Default::default()
            })
        },
    };

    session.send(CMD_GET_CUR_SCENE_INFO_SC_RSP, resp).await?;

    Ok(())
}

lazy_static! {
    static ref NEXT_SCENE_SAVE: Mutex<u64> = Mutex::new(0);
}

async fn load_scene(json: &mut JsonData) -> Result<SceneInfo> {
    let enterance = GAME_RESOURCES
        .map_entrance
        .get(&json.scene.entry_id)
        .ok_or_else(|| anyhow::format_err!("Map Entrance Not Found"))?;

    let _plane = GAME_RESOURCES
        .maze_plane
        .get(&enterance.plane_id)
        .ok_or_else(|| anyhow::format_err!("Map Plane Not Found"))?;

    let group_config = GAME_RESOURCES
        .level_group
        .get(&format!("P{}_F{}", enterance.plane_id, enterance.floor_id))
        .ok_or_else(|| anyhow::format_err!("Group Config Not Found"))?;

    let mut position = json.position.clone();
    let teleport_id = Option::<u32>::None;
    if let Some(teleport_id) = teleport_id {
        if let Some(teleport) = group_config.teleports.get(&teleport_id) {
            let anchor = group_config
                .group_items
                .get(&teleport.anchor_group_id.unwrap_or_default())
                .and_then(|v| v.anchors.get(&teleport.anchor_id.unwrap_or_default()));
            if let Some(anchor) = anchor {
                position.x = (anchor.pos_x * 1000f64) as i32;
                position.y = (anchor.pos_y * 1000f64) as i32;
                position.z = (anchor.pos_z * 1000f64) as i32;
                position.rot_y = (anchor.rot_y * 1000f64) as i32;
            }
        }
    }

    let mut scene_info = SceneInfo {
        plane_id: json.scene.plane_id,
        floor_id: json.scene.floor_id,
        entry_id: json.scene.entry_id,
        game_mode_type: 1,
        ..Default::default()
    };

    let mut loaded_npc: Vec<u32> = vec![];
    let mut prop_entity_id = 1_000;
    let mut npc_entity_id = 20_000;
    let mut monster_entity_id = 30_000;

    for (group_id, group) in &group_config.group_items {
        let mut group_info = SceneGroupInfo {
            state: 1,
            group_id: *group_id,
            ..Default::default()
        };

        // Load Props
        for prop in &group.props {
            let prop_state = if prop.prop_state_list.contains(&PropState::CheckPointEnable) {
                PropState::CheckPointEnable
            } else {
                prop.state.clone()
            };

            prop_entity_id += 1;

            let prop_position = Position {
                x: (prop.pos_x * 1000f64) as i32,
                y: (prop.pos_y * 1000f64) as i32,
                z: (prop.pos_z * 1000f64) as i32,
                rot_y: (prop.rot_y * 1000f64) as i32,
            };

            let entity_info = SceneEntityInfo {
                inst_id: prop.id,
                group_id: prop.group_id,
                motion: Some(prop_position.to_motion()),
                prop: Some(ScenePropInfo {
                    prop_id: prop.prop_id,
                    prop_state: prop_state as u32,
                    ..Default::default()
                }),
                entity_id: prop_entity_id,
                ..Default::default()
            };

            group_info.entity_list.push(entity_info);
        }

        group_info.entity_list.push(SceneEntityInfo {
            entity_id: 1337,
            group_id: 186,
            inst_id: 300001,
            motion: Some(MotionInfo {
                // pos
                pos: Some(Vector {
                    x: json.position.x + 6,
                    y: json.position.y,
                    z: json.position.z + 6,
                }),
                // rot
                rot: Some(Vector {
                    ..Default::default()
                }),
            }),
            prop: Some(ScenePropInfo {
                prop_id: 808,
                prop_state: 1,
                ..Default::default()
            }),
            ..Default::default()
        });

        // Load NPCs
        for npc in &group.npcs {
            if loaded_npc.contains(&(npc.npcid)) || json.avatars.contains_key(&(npc.npcid)) {
                continue;
            }
            npc_entity_id += 1;
            loaded_npc.push(npc.npcid);

            let npc_position = Position {
                x: (npc.pos_x * 1000f64) as i32,
                y: (npc.pos_y * 1000f64) as i32,
                z: (npc.pos_z * 1000f64) as i32,
                rot_y: (npc.rot_y * 1000f64) as i32,
            };

            let info = SceneEntityInfo {
                inst_id: npc.id,
                group_id: npc.group_id,
                entity_id: npc_entity_id,
                motion: Some(npc_position.to_motion()),
                npc: Some(SceneNpcInfo {
                    npc_id: npc.npcid,
                    ..Default::default()
                }),
                ..Default::default()
            };

            group_info.entity_list.push(info);
        }

        for monster in &group.monsters {
            monster_entity_id += 1;
            let monster_position = Position {
                x: (monster.pos_x * 1000f64) as i32,
                y: (monster.pos_y * 1000f64) as i32,
                z: (monster.pos_z * 1000f64) as i32,
                rot_y: (monster.rot_y * 1000f64) as i32,
            };

            let npc_monster = SceneNpcMonsterInfo {
                monster_id: monster.npcmonster_id,
                event_id: monster.event_id,
                world_level: 6,
                ..Default::default()
            };

            let info = SceneEntityInfo {
                inst_id: monster.id,
                group_id: monster.group_id,
                entity_id: monster_entity_id,
                motion: Some(monster_position.to_motion()),
                npc_monster: Some(npc_monster),
                ..Default::default()
            };

            group_info.entity_list.push(info);
        }

        scene_info.scene_group_list.push(group_info);
    }

    // load player entity
    let mut player_group = SceneGroupInfo {
        state: 1,
        group_id: 0,
        ..Default::default()
    };
    for (_slot, avatar_id) in &json.lineups {
        player_group.entity_list.push(SceneEntityInfo {
            entity_id: 0,
            group_id: 0,
            inst_id: 0,
            motion: Some(MotionInfo {
                // pos
                pos: Some(Vector {
                    x: json.position.x,
                    y: json.position.y,
                    z: json.position.z,
                }),
                // rot
                rot: Some(Vector {
                    ..Default::default()
                }),
            }),
            actor: Some(SceneActorInfo {
                avatar_type: AvatarType::AvatarFormalType.into(),
                base_avatar_id: *avatar_id,
                map_layer: 2,
                uid: 1337,
            }),
            ..Default::default()
        });
    }
    scene_info.scene_group_list.push(player_group);

    tracing::info!("scene_info:{:#?}", scene_info);
    Ok(scene_info)
}

pub async fn on_scene_entity_move_cs_req(
    session: &mut PlayerSession,
    request: &SceneEntityMoveCsReq,
) -> Result<()> {
    let mut _player = JsonData::load().await;
    let mut timestamp = NEXT_SCENE_SAVE.lock().await;

    if util::cur_timestamp_ms() <= *timestamp {
        session
            .send(CMD_SCENE_ENTITY_MOVE_SC_RSP, Dummy::default())
            .await?;

        return Ok(());
    }

    // save every 5 sec
    *timestamp = util::cur_timestamp_ms() + (5 * 1000);

    for entity in &request.entity_motion_list {
        tracing::info!("entity:{:#?}", entity);
        if entity.entity_id != 0 {
            continue;
        }

        // if let Some(motion) = &entity.motion {
        //     if let Some(pos) = &motion.pos {
        //         _player.position.x = pos.x;
        //         _player.position.y = pos.y;
        //         _player.position.z = pos.z;
        //     }
        //     if let Some(rot) = &motion.rot {
        //         _player.position.rot_y = rot.y;
        //     }
        // }
    }

    //_player.save().await;
    session
        .send(CMD_SCENE_ENTITY_MOVE_SC_RSP, Dummy::default())
        .await
}
