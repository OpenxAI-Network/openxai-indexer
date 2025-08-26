use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::providers::DynProvider;
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        Database,
        deployment_signature::{
            DatabaseDeploymentSignature, DatabaseDeploymentSignaturePerDayCount,
        },
    },
    utils::{signature_validator::validate_signature, time::get_time_i64},
};

#[get("/deployment_signature/total")]
async fn get_total(database: web::Data<Database>) -> impl Responder {
    match DatabaseDeploymentSignature::get_count(&database).await {
        Ok(total) => HttpResponse::Ok().json(total),
        Err(e) => {
            log::error!("Fetching deployment signature count: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/deployment_signature/per_day")]
async fn get_per_day(database: web::Data<Database>) -> impl Responder {
    match DatabaseDeploymentSignaturePerDayCount::get_all(&database).await {
        Ok(per_day) => HttpResponse::Ok().json(per_day),
        Err(e) => {
            log::error!("Fetching deployment signature per day count: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/deployment_signature/latest/{app}")]
async fn get_latest(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let app = path.into_inner();
    match DatabaseDeploymentSignature::get_latest_by_app(&database, &app, 10).await {
        Ok(deployments) => HttpResponse::Ok().json(deployments),
        Err(e) => {
            log::error!("Fetching latest deployment signature for {app}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/deployment_signature/total/{app}")]
async fn get_app_total(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let app = path.into_inner();
    match DatabaseDeploymentSignature::get_count_by_app(&database, &app).await {
        Ok(total) => HttpResponse::Ok().json(total),
        Err(e) => {
            log::error!("Fetching deployment signature count for {app}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/deployment_signature/total/{app}/{version}")]
async fn get_app_version_total(
    database: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (app, version) = path.into_inner();
    match DatabaseDeploymentSignature::get_count_by_app_version(&database, &app, &version).await {
        Ok(total) => HttpResponse::Ok().json(total),
        Err(e) => {
            log::error!("Fetching deployment signature count for {app} {version}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeploymentSignature {
    pub xnode: String,
    pub app: String,
    pub version: String,
    pub deployer: Option<String>,
    pub signature: Option<String>,
}
#[post("/deployment_signature/upload")]
async fn post_upload(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<DeploymentSignature>,
) -> impl Responder {
    if let Some(deployer) = &data.deployer {
        match &data.signature {
            Some(signature) => {
                let message = format!(
                    "I just deployed {version} of {app} on OpenxAI Studio!",
                    version = data.version,
                    app = data.app
                );
                if !validate_signature(provider.get_ref(), deployer, &message, signature).await {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            None => {
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    let deployment_signature = DatabaseDeploymentSignature {
        xnode: data.xnode.clone(),
        app: data.app.clone(),
        version: data.version.clone(),
        deployer: data.deployer.clone(),
        signature: data.signature.clone(),
        date: get_time_i64(),
    };
    if let Err(e) = deployment_signature.insert(&database).await {
        log::error!("COULD NOT INSERT DEPLOYMENT SIGNATURE {deployment_signature:?}: {e}");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
