-- EVE static data (SDE) JSONL rows: full line JSON per `_key` for fast lookup and refresh on re-import.
CREATE TABLE IF NOT EXISTS EveSdeTypes (
  TypeId INTEGER PRIMARY KEY NOT NULL,
  Payload TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS EveSdeGroups (
  GroupId INTEGER PRIMARY KEY NOT NULL,
  Payload TEXT NOT NULL
);
