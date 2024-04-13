package lib

import (
	"encoding/json"
)

// Go nie ma wsparcia dla ENUM, trzeba się bawić w stałe liczbowe, których nazwy mogą konfliktować, więc trzeba je prefiksować

type TitleStatus int

func (t TitleStatus) MarshalJSON() ([]byte, error) {
	switch t {
	case TitleUnknown:
		return json.Marshal("Unknown")
	case TitleNotYetAired:
		return json.Marshal("NotYetAired")
	case TitleFinishedAiring:
		return json.Marshal("FinishedAiring")
	case TitleCurrentlyAiring:
		return json.Marshal("CurrentlyAiring")
	case TitleProposal:
		return json.Marshal("Proposal")
	}

	return nil, nil
}

func (t *TitleStatus) UnmarshalJSON(bytes []byte) error {
	var name string
	if err := json.Unmarshal(bytes, &name); err != nil {
		return err
	}

	switch name {
	case "Not yet aired", "UPCOMING", "NotYetAired":
		*t = TitleNotYetAired
	case "Finished Airing", "FINISHED", "FinishedAiring":
		*t = TitleFinishedAiring
	case "Currently Airing", "ONGOING", "CurrentlyAiring":
		*t = TitleCurrentlyAiring
	case "PROPOSAL", "Proposal":
		*t = TitleProposal
	default:
		*t = TitleUnknown
	}

	return nil
}

const (
	TitleUnknown TitleStatus = iota
	TitleNotYetAired
	TitleFinishedAiring
	TitleCurrentlyAiring
	TitleProposal
)

type AnimeType int

func (a AnimeType) MarshalJSON() ([]byte, error) {
	switch a {
	case AnimeUnknown:
		return json.Marshal("Unknown")
	case AnimeTV:
		return json.Marshal("TV")
	case AnimeOVA:
		return json.Marshal("OVA")
	case AnimeONA:
		return json.Marshal("ONA")
	case AnimeSpecial:
		return json.Marshal("Special")
	case AnimeMusic:
		return json.Marshal("Music")
	case AnimeMovie:
		return json.Marshal("Movie")
	}

	return nil, nil
}

func (a *AnimeType) UnmarshalJSON(bytes []byte) error {
	var name string
	if err := json.Unmarshal(bytes, &name); err != nil {
		return err
	}

	switch name {
	case "TV":
		*a = AnimeTV
	case "OVA":
		*a = AnimeOVA
	case "ONA":
		*a = AnimeONA
	case "Special", "SPECIAL":
		*a = AnimeSpecial
	case "Music":
		*a = AnimeMusic
	case "Movie", "MOVIE":
		*a = AnimeMovie
	default:
		*a = AnimeUnknown
	}

	return nil
}

const (
	AnimeUnknown AnimeType = iota
	AnimeTV
	AnimeOVA
	AnimeONA
	AnimeSpecial
	AnimeMusic
	AnimeMovie
)

type WatchStatus int

func (w WatchStatus) MarshalJSON() ([]byte, error) {
	switch w {
	case WatchUnknown:
		return json.Marshal("Unknown")
	case WatchCompleted:
		return json.Marshal("Completed")
	case WatchSkip:
		return json.Marshal("Skip")
	case WatchPlan:
		return json.Marshal("Plan")
	case WatchDropped:
		return json.Marshal("Dropped")
	case WatchInProgress:
		return json.Marshal("InProgress")
	case WatchHold:
		return json.Marshal("Hold")
	}

	return nil, nil
}

func (w *WatchStatus) UnmarshalJSON(bytes []byte) error {
	var name string
	if err := json.Unmarshal(bytes, &name); err != nil {
		return err
	}

	switch name {
	case "completed", "Completed":
		*w = WatchCompleted
	case "skip", "Skip":
		*w = WatchSkip
	case "plan", "Plan":
		*w = WatchPlan
	case "dropped", "Dropped":
		*w = WatchDropped
	case "in progress", "InProgress":
		*w = WatchInProgress
	case "hold", "Hold":
		*w = WatchHold
	default:
		*w = WatchUnknown
	}

	return nil
}

const (
	WatchUnknown WatchStatus = iota
	WatchCompleted
	WatchSkip
	WatchPlan
	WatchDropped
	WatchInProgress
	WatchHold
)
