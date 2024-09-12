use crate::web::common::serde_chrono::ApiDateTime;


#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Info {
    pub now: ApiDateTime,
    pub trace_id: TraceId,
    pub principal_id: Option<i64>,
}

pub async fn info<TNow: Now>(
    now: web::Data<TNow>,
    trace_id: web::ReqData<TraceId>,
    principal: web::ReqData<Option<Principal>>,
) -> ApiResult<Info> {
    let now = now.now();
    tracing::info!("info!");
    Ok(web::Json(Info {
        now: now.into(),
        trace_id: trace_id.into_inner(),
        principal_id: principal.as_ref().map(|v| v.id()),
    }))
}
