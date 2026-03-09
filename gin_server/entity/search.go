package entity

type SearchGroupResult struct {
	ID     string `json:"id"`
	Name   string `json:"name"`
	Avatar string `json:"avatar"`
	Status string `json:"status"`
}
type SearchUserResult struct {
	ID     string `json:"id"`
	Name   string `json:"name"`
	Avatar string `json:"avatar"`
	Status string `json:"status"`
}

type SearchResult struct {
	Groups []*SearchGroupResult `json:"groups"`
	Users  []*SearchUserResult  `json:"users"`
}
