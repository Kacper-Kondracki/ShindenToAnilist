export namespace db {
	
	export class AnimeSeason {
	    season: string;
	    year?: number;
	
	    static createFrom(source: any = {}) {
	        return new AnimeSeason(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.season = source["season"];
	        this.year = source["year"];
	    }
	}
	export class Anime {
	    sources: string[];
	    title: string;
	    type: number;
	    episodes: number;
	    status: number;
	    animeSeason: AnimeSeason;
	    picture: string;
	    thumbnail: string;
	    synonyms: string[];
	    relations: string[];
	    tags: string[];
	
	    static createFrom(source: any = {}) {
	        return new Anime(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.sources = source["sources"];
	        this.title = source["title"];
	        this.type = source["type"];
	        this.episodes = source["episodes"];
	        this.status = source["status"];
	        this.animeSeason = this.convertValues(source["animeSeason"], AnimeSeason);
	        this.picture = source["picture"];
	        this.thumbnail = source["thumbnail"];
	        this.synonyms = source["synonyms"];
	        this.relations = source["relations"];
	        this.tags = source["tags"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

export namespace main {
	
	export class ConvertResultJSON {
	    status: string;
	    successCount: number;
	    multipleCount: number;
	    failCount: number;
	    successAnime: searcher.SearchSuccess[];
	    multipleAnime: searcher.SearchMultiple[];
	    failAnime: searcher.SearchFail[];
	
	    static createFrom(source: any = {}) {
	        return new ConvertResultJSON(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.status = source["status"];
	        this.successCount = source["successCount"];
	        this.multipleCount = source["multipleCount"];
	        this.failCount = source["failCount"];
	        this.successAnime = this.convertValues(source["successAnime"], searcher.SearchSuccess);
	        this.multipleAnime = this.convertValues(source["multipleAnime"], searcher.SearchMultiple);
	        this.failAnime = this.convertValues(source["failAnime"], searcher.SearchFail);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

export namespace searcher {
	
	export class SearchFail {
	    search_anime?: any;
	
	    static createFrom(source: any = {}) {
	        return new SearchFail(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.search_anime = source["search_anime"];
	    }
	}
	export class SearchMultiple {
	    search_anime?: any;
	    db_anime?: db.Anime[];
	
	    static createFrom(source: any = {}) {
	        return new SearchMultiple(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.search_anime = source["search_anime"];
	        this.db_anime = this.convertValues(source["db_anime"], db.Anime);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class SearchSuccess {
	    search_anime?: any;
	    db_anime: db.Anime;
	
	    static createFrom(source: any = {}) {
	        return new SearchSuccess(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.search_anime = source["search_anime"];
	        this.db_anime = this.convertValues(source["db_anime"], db.Anime);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

