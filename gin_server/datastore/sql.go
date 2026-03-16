package datastore

const (
	CreateUsersTableSQL = `CREATE TABLE IF NOT EXISTS users (
	id UUID PRIMARY KEY,
	name TEXT NOT NULL,
	avatar TEXT,
	status TEXT,
	create_time BIGINT,
	update_time BIGINT
);`

	CreateGroupsTableSQL = `CREATE TABLE IF NOT EXISTS groups (
	id UUID PRIMARY KEY,
	name TEXT NOT NULL,
	avatar TEXT,
	create_time BIGINT,
	update_time BIGINT
);`

	CreateFriendsTableSQL = `CREATE TABLE IF NOT EXISTS friends (
	id UUID PRIMARY KEY,
	user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	friend_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	create_time BIGINT,
	update_time BIGINT,
	UNIQUE (user_id, friend_id)
);`

	CreateGroupMembersTableSQL = `CREATE TABLE IF NOT EXISTS group_members (
	id UUID PRIMARY KEY,
	group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
	user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	user_type TEXT,
	status TEXT,
	create_time BIGINT,
	update_time BIGINT,
	UNIQUE (group_id, user_id)
);`

	CreateGroupHistoryTableSQL = `CREATE TABLE IF NOT EXISTS group_history (
	id UUID PRIMARY KEY,
	group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
	sender_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	message TEXT NOT NULL,
	time TEXT,
	files TEXT[],
	create_time BIGINT
);`
)
