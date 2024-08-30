#[get("/")]
pub fn query_bucket_page() -> Result<Json<HashMap<String, Bucket>>, HttpErrorJson> {
    
}


#[get("/<bucket_id>")]
pub fn query_bucket_detail() -> Result<Bucket,HttpErrorJson>{

}