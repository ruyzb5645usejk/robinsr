use crate::util;

use super::*;

pub async fn on_get_basic_info_cs_req(
    session: &mut PlayerSession,
    _body: &GetBasicInfoCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_BASIC_INFO_SC_RSP,
            GetBasicInfoScRsp {
                cur_day: 1,
                exchange_times: 0,
                retcode: 0,
                next_recover_time: 2281337,
                week_cocoon_finished_count: 0,
                ..Default::default()
            },
        )
        .await
}

pub async fn on_player_heart_beat_cs_req(
    session: &mut PlayerSession,
    body: &PlayerHeartBeatCsReq,
) -> Result<()> {
    session
        .send(
            CMD_PLAYER_HEART_BEAT_SC_RSP,
            PlayerHeartBeatScRsp {
                retcode: 0,
                client_time_ms: body.client_time_ms,
                server_time_ms: util::cur_timestamp_ms(),
                download_data: Some(ClientDownloadData {//更改versiontxt和uid
                    version: 51,
                    time: util::cur_timestamp_ms() as i64,
                    data: rbase64::decode("Q1MuVW5pdHlFbmdpbmUuR2FtZU9iamVjdC5GaW5kKCJVSVJvb3QvQWJvdmVEaWFsb2cvQmV0YUhpbnREaWFsb2coQ2xvbmUpIik6R2V0Q29tcG9uZW50SW5DaGlsZHJlbih0eXBlb2YoQ1MuUlBHLkNsaWVudC5Mb2NhbGl6ZWRUZXh0KSkudGV4dCA9ICI8Y29sb3I9IzAwRkZGRj48Yj4/Pz88L2I+PC9jb2xvcj4iCkNTLlVuaXR5RW5naW5lLkdhbWVPYmplY3QuRmluZCgiVmVyc2lvblRleHQiKTpHZXRDb21wb25lbnRJbkNoaWxkcmVuKHR5cGVvZihDUy5SUEcuQ2xpZW50LkxvY2FsaXplZFRleHQpKS50ZXh0ID0gIjxjb2xvcj0jRkYxNDkzPkNOQkVUQVdpbjIuNC41WDwvY29sb3I+Ig==").unwrap()
                }),
            },
        )
        .await
}
