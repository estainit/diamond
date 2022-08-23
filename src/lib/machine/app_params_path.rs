use crate::AppParams;

impl AppParams {


    //old_name_was getHDPath
    pub fn clone_path(&self) -> String
    {
        if self.id() == 0
        {
            return self.root_path();
        }
        return format!("{}/{}", self.root_path(), self.id());
    }


    //old_name_was getReportsPath
    pub fn reports_path(&self) -> String
    {
        return self.clone_path() + &"/reports";
    }

    //old_name_was getInboxPath
    pub fn inbox_path(&self) -> String
    {
        return self.clone_path() + &"/inbox";
    }

    //old_name_was getOutboxPath
    pub fn outbox_path(&self) -> String
    {
        return self.clone_path() + &"/outbox";
    }

    //old_name_was getReportsPath
    pub fn logs_path(&self) -> String
    {
        return self.clone_path() + &"/logs";
    }

    //old_name_was getCachePath
    pub fn cache_path(&self) -> String
    {
        return self.clone_path() + &"/cache_files";
    }

    //old_name_was getDAGBackup
    pub fn dag_backup(&self) -> String
    {
        return self.clone_path() + &"/dag_backup";
    }
}