-- Add migration script here
CREATE TABLE IF NOT EXISTS Events (
    ID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    kind varchar(35) NOT NULL,
    timestamp varchar(100) NOT NULL,
    device varchar(100),
    data varchar(255)
);