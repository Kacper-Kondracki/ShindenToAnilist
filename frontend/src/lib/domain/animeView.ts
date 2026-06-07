const missingValueText = "Brak danych";

const animeTypeLabels: Record<string, string> = {
  tv: "Serial TV",
  movie: "Film",
  ova: "OVA",
  ona: "ONA",
  special: "Odcinek specjalny",
  unknown: "Nieznany typ",
};

const animeStatusLabels: Record<string, string> = {
  finished: "Zakończone",
  ongoing: "Emitowane",
  upcoming: "Zapowiedziane",
  unknown: "Nieznany status",
};

const seasonLabels: Record<string, string> = {
  winter: "Zima",
  spring: "Wiosna",
  summer: "Lato",
  fall: "Jesień",
  unknown: "Nieznany sezon",
};

export function formatPremiereYear(premiereDate: string | null) {
  if (!premiereDate) {
    return "Nieznany rok";
  }

  return premiereDate.slice(0, 4);
}

export function formatYear(year: number | null) {
  return year === null ? missingValueText : String(year);
}

export function formatEpisodeCount(episodeCount: number | null) {
  if (episodeCount === null || episodeCount <= 0) {
    return missingValueText;
  }

  return String(episodeCount);
}

export function formatEpisodeDuration(duration: number | null) {
  if (duration === null || duration <= 0) {
    return missingValueText;
  }

  return `${duration} min`;
}

export function translateAnimeType(animeType: string) {
  return animeTypeLabels[animeType.toLowerCase()] ?? animeType;
}

export function translateAnimeStatus(animeStatus: string) {
  return animeStatusLabels[animeStatus.toLowerCase()] ?? animeStatus;
}

export function translateSeason(season: string) {
  if (!season) {
    return missingValueText;
  }

  return seasonLabels[season.toLowerCase()] ?? season;
}
