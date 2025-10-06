import { useState } from "react";
import { Search as SearchIcon, Music } from "lucide-react";
import { Input } from "@/components/ui/input";
import { GlassCard } from "@/components/GlassCard";

const Search = () => {
  const [searchQuery, setSearchQuery] = useState("");

  return (
    <div className="min-h-screen pt-32 pb-12 px-6">
      <div className="container mx-auto max-w-6xl">
        <div className="text-center mb-12 animate-fade-in">
          <h1 className="font-serif text-5xl font-bold mb-4 text-gold">
            Search Songs
          </h1>
          <p className="text-muted-foreground">
            Find your favorite tracks
          </p>
        </div>

        <div className="mb-8 animate-fade-in-up">
          <div className="relative">
            <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
            <Input
              type="text"
              placeholder="Search by song, artist, or album..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-12 h-12 glass-card border-border/50 focus:border-gold/50 focus:ring-gold/20"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-fade-in-up">
          {[1, 2, 3, 4, 5, 6].map((i) => (
            <GlassCard key={i} className="group cursor-pointer">
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 rounded bg-gradient-to-br from-gold-dark to-gold flex items-center justify-center flex-shrink-0">
                  <Music className="h-8 w-8 text-background" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-lg truncate group-hover:text-gold transition-colors">
                    Song Title {i}
                  </h3>
                  <p className="text-sm text-muted-foreground truncate">
                    Artist Name
                  </p>
                  <p className="text-xs text-muted-foreground/70">
                    Album Name
                  </p>
                </div>
              </div>
            </GlassCard>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Search;
