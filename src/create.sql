CREATE TABLE IF NOT EXISTS tasks (
	id integer PRIMARY KEY AUTOINCREMENT,
	description text NOT NULL,
	complete boolean NOT NULL,
	parent integer,
	FOREIGN KEY(parent) REFERENCES tasks(id) ON DELETE NO ACTION
);

CREATE TRIGGER IF NOT EXISTS no_self_parenting
BEFORE INSERT ON tasks
FOR EACH ROW
WHEN NEW.parent IS NOT NULL AND (SELECT COUNT(1) FROM tasks WHERE NEW.parent = tasks.id) =	0
BEGIN
    SELECT RAISE(FAIL, "FOREIGN KEY constraint failed");
END;
