extern crate postgres;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::thread;
use postgres::{Client, NoTls};
use crate::lib::{constants};
use crate::lib::dlog::dlog;
use crate::{application, CMachine, dbhandler, machine};
use crate::lib::database::init_psql::{psql_init_query, psql_tables_list};
use crate::lib::database::init_psql_dev::psql_init_query_dev;

pub struct DBHandler {
    pub m_db_host: String,
    pub m_db_name: String,
    pub m_db_user: String,
    pub m_db_pass: String,
    pub m_current_clone: i8,
    pub m_db: Client,
}

impl DBHandler {
    pub(crate) fn new() -> DBHandler {
        eprintln!("New DBHandler is going to be created.");

        let db = DBHandler {
            m_db_host: "".to_string(),
            m_db_name: "".to_string(),
            m_db_user: "".to_string(),
            m_db_pass: "".to_string(),
            m_current_clone: 0,
            m_db: get_connection(),
        };

        eprintln!("New DBHandler was create.");

        return db;
    }
}

pub fn get_connection() -> Client {
    let db_host: String = application().db_host();
    let mut db_name: String = application().db_name();
    let db_user: String = application().db_user();
    let db_pass: String = application().db_pass();

    if application().id() > 0 {
        db_name = format!("{}{}", db_name, &application().id());
    }

    let mut connection_str = format!(
        "host={} dbname={} user={}",
        db_host,
        db_name,
        db_user
    );
    println!("Establish db connection for client {} => {}", &application().id(), connection_str);

    connection_str = format!("{} password={}", connection_str, db_pass);
    application().set_db_connected(true);
    return Client::connect(&connection_str, NoTls).unwrap();
}

//old_name_was initDb
pub fn maybe_initialize_db() -> (bool, String)
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

        if is_created
        {
            application().set_db_initialized(true);
            return (true, "connected".to_string());
        }

        let creation_status = create_tables_in_psql();
        application().set_db_initialized(creation_status);
        return (creation_status, "connected".to_string());
    }
    return (false, "Unknown db agent!".to_string());
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
    println!("Create tables in PSQL");
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

    println!("PSQL tables are created.");
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