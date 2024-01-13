-- Add migration script here
CREATE TABLE todos (
  id uuid,
  title text not null,
  status boolean not null,

  PRIMARY KEY (id)
);
