use actix_web::web;

use crate::{
    utils::{time::Time, trace_id::TraceId},
    web::{
        app_data::AppData,
        common::{api_result::ApiResult, serde_chrono::ApiDateTime},
    },
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Info {
    pub now: ApiDateTime,
    pub trace_id: TraceId,
}

pub async fn info<D: AppData>(
    data: web::Data<D>,
    trace_id: web::ReqData<TraceId>,
) -> ApiResult<Info> {
    let now = data.time().now();
    Ok(web::Json(Info {
        now: now.into(),
        trace_id: trace_id.into_inner(),
    }))
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::{test::*, utc, web::trace_id};
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn info() {
        test(|ctx| async move {
            ctx.logs().write_always();

            let server = ctx.run_server().await;
            let now = utc!(2023, 12, 1, 2, 3, 4, 125);
            ctx.time().set(now);

            // act
            let response = server.client().get("/api/info/v1").send().await;

            // assert
            let info = response.unwrap::<Info>();
            let header_trace_id: TraceId = response
                .headers
                .get(trace_id::HEADER_NAME)
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap();

            assert_eq!(info.trace_id, header_trace_id);
            assert_eq!(*info.now, now);
        });
    }
}
