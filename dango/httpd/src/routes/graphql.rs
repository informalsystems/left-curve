use {
    crate::graphql::AppSchema,
    actix_web::{HttpRequest, HttpResponse, Resource, web},
    async_graphql::Schema,
    async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription},
    indexer_httpd::routes::graphql::graphiql_playgound,
};

pub fn graphql_route() -> Resource {
    web::resource("/graphql")
        .route(web::post().to(graphql_index))
        .route(
            web::get()
                .guard(actix_web::guard::Header("upgrade", "websocket"))
                .to(graphql_ws),
        )
        .route(web::get().to(graphiql_playgound))
}

#[tracing::instrument(name = "graphql::graphql_index", skip_all)]
pub(crate) async fn graphql_index(
    schema: web::Data<AppSchema>,
    _req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let request = gql_request.into_inner();

    schema.execute(request).await.into()
}

#[tracing::instrument(name = "graphql::graphql_ws", skip_all)]
pub(crate) async fn graphql_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
