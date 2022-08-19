extern crate postgres;

use std::thread;
use postgres::{Client, NoTls};
use crate::lib::{constants};
use crate::lib::dlog::dlog;
use crate::{CMachine, dbhandler, machine};
use crate::lib::database::init_psql::{psql_init_query, psql_tables_list};
use crate::lib::database::init_psql_dev::psql_init_query_dev;

pub struct DBHandler {
    pub m_db: Client,
}

impl DBHandler {
    pub(crate) fn new() -> DBHandler {
        DBHandler {
            m_db: get_connection(0)
        }
    }
}

pub fn getConnection() -> Client {
    let mut database_name: String = constants::psql_db::DB_NAME.to_string();
    let clone_id = 0;//machine().get_app_clone_id();
    if clone_id > 0 {
        database_name += &clone_id.to_string();
    }

    let db_connection_full_name = database_name + "_" + "CDB_" + &format!("{:?}", thread::current().id());

    dlog(
        &format!("db_connection_full_name to create({})", db_connection_full_name),
        constants::Modules::Sql,
        constants::SecLevel::Trace);
    let mut connection_str = format!(
        "host={host} user={user} password={password}",
        host = constants::psql_db::DB_HOST,
        user = constants::psql_db::DB_USER,
        password = constants::psql_db::DB_PASS,
    );
    return Client::connect(&connection_str, NoTls).unwrap();

    // let mut client = Client::connect("host=localhost user=diamond password=diamondpass", NoTls).unwrap();
    //
    // return client;
//     client.batch_execute("
//     CREATE TABLE person (
//         id      SERIAL PRIMARY KEY,
//         name    TEXT NOT NULL,
//         data    BYTEA
//     )
// ");
//
//     let name = "Ferris";
//     let data = None::<&[u8]>;
//     client.execute(
//         "INSERT INTO person (name, data) VALUES ($1, $2)",
//         &[&name, &data],
//     );
//
//     for row in client.query("SELECT id, name, data FROM person", &[]) {
//         let id: i32 = row.get(0);
//         let name: &str = row.get(1);
//         let data: Option<&[u8]> = row.get(2);
//
//         println!("found person: {} {} {:?}", id, name, data);
//     }
}

pub fn get_connection(clone_id: i8) -> Client {
    let mut database_name: String = constants::psql_db::DB_NAME.to_string();
    // let clone_id = 0;//machine().get_app_clone_id();
    if clone_id > 0 {
        database_name = format!("{}{}", database_name, clone_id);
    }
    println!("Stablish db connection for client {} database_name: {}", clone_id, database_name);

    let mut connection_str = format!(
        "host={host} dbname={database} user={user} password={password}",
        host = constants::psql_db::DB_HOST,
        user = constants::psql_db::DB_USER,
        password = constants::psql_db::DB_PASS,
        database = database_name,
    );
    return Client::connect(&connection_str, NoTls).unwrap();
}

//old_name_was initDb
pub fn maybe_initialize_db<'a>(machine: &mut CMachine) -> (bool, String)
{
    if constants::DATABASAE_AGENT == "psql"
    {
        let is_created: bool = match dbhandler().m_db.query("SELECT * FROM c_blocks limit 1;", &[]) {
            Ok(rows) => {
                true
            }
            Err(e) => {
                println!("No table existed, so create it");
                false
            }
        };

        return if is_created
        {
            machine.m_is_db_connected = true;
            machine.m_is_db_initialized = true;
            (true, "connected".to_string())
        } else {
            let creation_status = create_tables_in_psql();
            machine.m_is_db_connected = creation_status;
            machine.m_is_db_initialized = creation_status;
            if machine.m_is_db_initialized
            {
                machine.maybe_add_seed_neighbors();
            }
            (creation_status, "connected".to_string())
        };
    }
    return (false, "Unknown db agent!".to_string());
}

pub fn s_databases<'l>() -> Vec<&'l str> {
    return vec![
        "db_comen_blocks",
        "db_comen_block_ext_info",
        "db_comen_spendable_coins",
        "db_comen_spent_coins",
        "db_comen_general",
        "db_comen_wallets"];
}

/*

bool DbHandler::IcloseConnections()
{

    for (String connection_key: m_sqlite_thread_connections.keys())
    {
      m_psql_thread_connections[connection_key]->close();
      delete m_psql_thread_connections[connection_key];
    }

  return true;
}

std::tuple<bool, String, QSqlDatabase> DbHandler::IconnectToSQLITE(String database_name)
{
  /**
   * @brief divided databases, in order to address scaleability
   * comen_blocks:
   * comen_block_ext_info:
   * comen_spendable_coins:
   * comen_spent_coins:
   * comen_general: // it will be devided to smaller databases (e.g wiki database, Demos database, version control database, inames database, signals database, logs databases, etc...
   * comen_wallets:
   *
   */
  auto thread_name = "CDB_" + String::number((quint64)QThread::currentThread(), 16);
//  CLog::log("thread_name to check(" + thread_name + ")", "sql", "trace");

  if (database_name == "")
    database_name = "db_comen_general";

  String db_connection_full_name = database_name + "_" + thread_name;
  if (m_sqlite_thread_connections.keys().contains(db_connection_full_name))
    return {true, db_connection_full_name + " Already connected to Database", m_sqlite_thread_connections[db_connection_full_name]};

  CLog::log("db_connection_full_name to create(" + db_connection_full_name + ")", "sql", "trace");

  QSqlDatabase the_database;

  const String SQLITE_DRIVER {"QSQLITE"};
  if (!QSqlDatabase::isDriverAvailable(SQLITE_DRIVER))
    cutils::exiter("SQLITE_DRIVER Failed to open database", 23);

  the_database = QSqlDatabase::addDatabase(SQLITE_DRIVER, db_connection_full_name);
  QSqlQuery thread_q = QSqlQuery(the_database);
  the_database.setDatabaseName(QCoreApplication::applicationDirPath() + QDir::separator() + database_name + ".dat");
  if (!the_database.open())
  {
    qDebug() << "Failed to open database" << the_database.lastError();
    cutils::exiter("SQLITE_DRIVER Failed to open database", 23);
  }

  if (!CMachine::databases_are_created())
  {
    bool res = thread_q.exec("SELECT name FROM sqlite_master WHERE type='table' AND name='c_blocks'");
    if (!res || !thread_q.last())
      CMachine::set_databases_are_created(createTablesSQLITE());
  }

  m_sqlite_thread_connections[db_connection_full_name] = the_database;

  return
  {
    true,
    "",
    m_sqlite_thread_connections[db_connection_full_name]
  };

}
*/

//old_name_was emptyDB
pub fn empty_db(machine: &mut CMachine) -> bool
{
    for a_table in psql_tables_list()
    {
        let query_string = format!("DELETE FROM {}", a_table);
        dlog(
            &format!("Cleaning table: {}", a_table),
            constants::Modules::Sql,
            constants::SecLevel::Info);
        let q_err_num: u64 = dbhandler().m_db.execute(&query_string, &[]).unwrap();
        if q_err_num > 0
        { return false; }
    }
    return true;
}

//old_name_was createTablesPSQL
pub fn create_tables_in_psql() -> bool
{
    println!("create_ tables _in _psql");
    for a_query in psql_init_query()
    {
        let mut qstr = a_query.clone();
        dlog(
            &format!("Creating table: {}", qstr),
            constants::Modules::Sql,
            constants::SecLevel::Info);
        let q_err_num: u64 = dbhandler().m_db.execute(qstr, &[]).unwrap();
        if q_err_num > 0
        { return false; }
    }

    // create developers tables
    for a_query in psql_init_query_dev()
    {
        let mut qstr = a_query.clone();
        dlog(
            &format!("Creating table: {}", qstr),
            constants::Modules::Sql,
            constants::SecLevel::Info);

        let q_err_num: u64 = dbhandler().m_db.execute(qstr, &[]).unwrap();
        if q_err_num > 0
        { return false; }
    }

    println!("@@@@@@@@@@");
    return true;
}

/*
bool DbHandler::createTablesSQLITE()
{
  const String SQLITE_DRIVER {"QSQLITE"};
  for (String a_db: s_databases)
  {
    CLog::log("Creating tables for database(" + a_db + ")", "sql");
    QSqlDatabase tmp_database = QSqlDatabase::addDatabase(SQLITE_DRIVER, a_db);
    QSqlQuery tmp_q = QSqlQuery(tmp_database);
    tmp_database.setDatabaseName(QCoreApplication::applicationDirPath() + QDir::separator() + a_db + ".dat");
    if (!tmp_database.open())
      cutils::exiter("failed to open database(" + a_db + ")", 789);


    // FIXME: create only needed tables
    // create essential tables
    for (std::string aQuery : sqlite_init_query)
    {
      String qstr = String::fromStdString(aQuery);
      bool res = tmp_q.exec(qstr);
      if (!res){
          std::cout << std::endl << "Error! " << aQuery;
      }
      tmp_q.finish();
    }

    // create developers tables
    for (std::string aQuery : sqlite_init_query_dev)
    {
      String qstr = String::fromStdString(aQuery);
      bool res = tmp_q.exec(qstr);
      if (!res){
          std::cout << std::endl << "Error! " << aQuery;
      }
      tmp_q.finish();
    }
  }
  return true;
}
*/


/*

QSqlDatabase DbHandler::IgetDB(const String& database_name)
{
  auto[status, msg, db] = connectToSQLITE(database_name);
  Q_UNUSED(msg);
  if (status)
    return db;

  auto thread_name = "CDB_" + String::number((quint64)QThread::currentThread(), 16);
  cutils::exiter("Invalid thread database request sqlite" + thread_name, 14);
  return QSqlDatabase {};
};

QSqlDatabase* DbHandler::IgetPSQLDB(const String& database_name)
{
  auto[status, db] = connectToPSQL(database_name);
  if (status)
    return db;

  auto thread_name = "CDB_" + String::number((quint64)QThread::currentThread(), 16);
  cutils::exiter("Invalid thread database request psql " + thread_name, 14);
  return db;
};

int DbHandler::initValues()
{
    return 1;
}

 */