CREATE TABLE tasks (
	id integer PRIMARY KEY AUTOINCREMENT,
	description text NOT NULL,
	complete boolean NOT NULL,
	parent integer,
	FOREIGN KEY(parent) REFERENCES tasks(id) ON DELETE CASCADE
);
