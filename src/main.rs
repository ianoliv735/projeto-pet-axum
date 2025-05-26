use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use axum_server::Server;
use serde::Deserialize;
use std::{net::SocketAddr, fs, sync::Arc};
use tower_http::services::ServeDir;
use rusqlite::params;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

type DbPool = Pool<SqliteConnectionManager>;

#[tokio::main]
async fn main() {
    // Imprime o diretório atual para ajudar no diagnóstico
    println!("Diretório atual: {:?}", std::env::current_dir().unwrap());

    
    let manager = SqliteConnectionManager::file("projeto_rust.db");
    let pool = Pool::new(manager).expect("Falha ao criar pool");

    // Cria as tabelas na inicialização
    {
        let conn = pool.get().unwrap();
        criar_tabelas(&conn).unwrap();
    }

    let static_service = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_form))
        .route("/banho_tosa", get(banho_tosa_form))
        .route("/consulta", get(consulta_form))
        .route("/login", post(login_submit))
        .route("/banho_tosa", post(banho_tosa_submit))
        .route("/consulta", post(consulta_submit))
        .nest_service("/static", static_service)
        .with_state(Arc::new(pool)); // Arc permite clone seguro

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor rodando em http://{}", addr);

    Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn criar_tabelas(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS banho_tosa (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL,
            cpf TEXT NOT NULL,
            celular TEXT NOT NULL,
            nome_pet TEXT NOT NULL,
            motivo TEXT NOT NULL,
            data TEXT NOT NULL,
            horario TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS consulta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL,
            cpf TEXT NOT NULL,
            celular TEXT NOT NULL,
            pet TEXT NOT NULL,
            motivo TEXT NOT NULL,
            data TEXT NOT NULL,
            horario TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

async fn login_form() -> Html<String> {
    let path = "templates/login.html";
    match fs::read_to_string(path) {
        Ok(contents) => Html(contents),
        Err(e) => Html(format!("<h1>Erro ao carregar {}: {}</h1>", path, e)),
    }
}

async fn index() -> Html<String> {
    let path = "templates/index.html";
    match fs::read_to_string(path) {
        Ok(contents) => Html(contents),
        Err(e) => Html(format!("<h1>Erro ao carregar {}: {}</h1>", path, e)),
    }
}

async fn banho_tosa_form() -> Html<String> {
    let path = "templates/banho_tosa.html";
    match fs::read_to_string(path) {
        Ok(contents) => Html(contents),
        Err(e) => Html(format!("<h1>Erro ao carregar {}: {}</h1>", path, e)),
    }
}

async fn consulta_form() -> Html<String> {
    let path = "templates/consulta.html";
    match fs::read_to_string(path) {
        Ok(contents) => Html(contents),
        Err(e) => Html(format!("<h1>Erro ao carregar {}: {}</h1>", path, e)),
    }
}

#[derive(Deserialize, Debug)]
struct LoginData {
    email: String,
    senha: String,
    lembrar: Option<String>,
}

#[derive(Deserialize, Debug)]
struct BanhoTosaData {
    nome: String,
    cpf: String,
    celular: String,
    nome_pet: String,
    motivo: String,
    data: String,
    horario: String,   
}

#[derive(Deserialize, Debug)]
struct ConsultaData {
    nome: String,
    cpf: String,
    celular: String,
    pet: String,
    motivo: String,
    data: String,
    horario: String,
}

async fn login_submit(Form(data): Form<LoginData>) -> impl IntoResponse {
    println!("Login recebido: {:?}", data);
    Html("<h1>Login recebido com sucesso!</h1><p><a href=\"/\">Voltar</a></p>".to_string())
}

async fn banho_tosa_submit(
    State(pool): State<Arc<DbPool>>,
    Form(data): Form<BanhoTosaData>,
) -> impl IntoResponse {
    // Clone os campos necessários antes de mover data
    let nome = data.nome.clone();
    let cpf = data.cpf.clone();
    let celular = data.celular.clone();
    let nome_pet = data.nome_pet.clone();
    let motivo = data.motivo.clone();
    let data_banho = data.data.clone();
    let horario_banho = data.horario.clone();

    let pool = pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO banho_tosa (nome, cpf, celular, nome_pet, motivo, data, horario) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![data.nome, data.cpf, data.celular, data.nome_pet, data.motivo, data.data, data.horario],
        )
    }).await;

    match result {
        Ok(Ok(_)) => Html(format!(
            "<h1>Agendamento de Banho e Tosa salvo com sucesso!</h1>
             <p><strong>Nome:</strong> {}</p>
             <p><strong>CPF:</strong> {}</p>
             <p><strong>Celular:</strong> {}</p>
             <p><strong>Nome Pet:</strong> {}</p>
             <p><strong>Motivo:</strong> {}</p>
             <p><strong>Data:</strong> {}</p>
             <p><strong>Horário:</strong> {}</p>
             <p><a href=\"/\">Voltar</a></p>",
            nome, cpf, celular, nome_pet, motivo, data_banho, horario_banho
        )),
        _ => Html("<h1>Erro ao salvar Banho e Tosa.</h1><p><a href=\"/\">Voltar</a></p>".to_string()),
    }
}


async fn consulta_submit(
    State(pool): State<Arc<DbPool>>,
    Form(data): Form<ConsultaData>,
) -> impl IntoResponse {
    // Clone os campos necessários antes de mover data
    let nome = data.nome.clone();
    let cpf = data.cpf.clone();
    let celular = data.celular.clone();
    let pet = data.pet.clone();
    let motivo = data.motivo.clone();
    let data_consulta = data.data.clone();
    let horario_consulta = data.horario.clone();

    let pool = pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO consulta (nome, cpf, celular, pet, motivo, data, horario) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![data.nome, data.cpf, data.celular, data.pet, data.motivo, data.data, data.horario],
        )
    }).await;

    match result {
        Ok(Ok(_)) => Html(format!(
            "<h1>Agendamento de Consulta salvo com sucesso!</h1>
             <p><strong>Nome:</strong> {}</p>
             <p><strong>CPF:</strong> {}</p>
             <p><strong>Celular:</strong> {}</p>
             <p><strong>Pet:</strong> {}</p>
             <p><strong>Motivo:</strong> {}</p>
             <p><strong>Data:</strong> {}</p>
             <p><strong>Horário:</strong> {}</p>
             <p><a href=\"/\">Voltar</a></p>",
            nome, cpf, celular, pet, motivo, data_consulta, horario_consulta
        )),
        _ => Html("<h1>Erro ao salvar agendamento.</h1><p><a href=\"/\">Voltar</a></p>".to_string()),
    }
}