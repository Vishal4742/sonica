import { History, Heart, TrendingUp, Music2 } from "lucide-react";
import { GlassCard } from "@/components/GlassCard";

const Profile = () => {
  const stats = [
    { label: "Songs Recognized", value: "127", icon: Music2, color: "text-gold" },
    { label: "Favorites", value: "42", icon: Heart, color: "text-gold-light" },
    { label: "This Week", value: "+18", icon: TrendingUp, color: "text-gold-dark" },
  ];

  return (
    <div className="min-h-screen pt-32 pb-12 px-6">
      <div className="container mx-auto max-w-6xl">
        <div className="text-center mb-12 animate-fade-in">
          <div className="w-24 h-24 mx-auto mb-4 rounded-full bg-gradient-to-br from-gold-dark via-gold to-gold-light flex items-center justify-center">
            <span className="text-4xl font-bold text-background">M</span>
          </div>
          <h1 className="font-serif text-4xl font-bold mb-2 text-gold">Music Lover</h1>
          <p className="text-muted-foreground">Music enthusiast</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12 animate-fade-in-up">
          {stats.map((stat, i) => (
            <GlassCard key={i} className="text-center">
              <stat.icon className={`h-8 w-8 mx-auto mb-3 ${stat.color}`} />
              <div className="text-3xl font-bold mb-1 text-gold">{stat.value}</div>
              <div className="text-sm text-muted-foreground">{stat.label}</div>
            </GlassCard>
          ))}
        </div>

        <div className="space-y-6 animate-fade-in-up">
          <GlassCard>
            <div className="flex items-center gap-3 mb-6">
              <History className="h-6 w-6 text-gold" />
              <h2 className="font-serif text-2xl font-bold">Recent Recognition History</h2>
            </div>
            
            <div className="space-y-4">
              {[1, 2, 3, 4, 5].map((i) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-4 rounded-xl bg-muted/20 hover:bg-muted/40 transition-colors cursor-pointer"
                >
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 rounded bg-gradient-to-br from-gold-dark to-gold flex items-center justify-center flex-shrink-0">
                      <Music2 className="h-6 w-6 text-background" />
                    </div>
                    <div>
                      <h3 className="font-semibold">Amazing Song Title</h3>
                      <p className="text-sm text-muted-foreground">Artist Name • Album</p>
                    </div>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {i} hours ago
                  </div>
                </div>
              ))}
            </div>
          </GlassCard>

          <GlassCard>
            <div className="flex items-center gap-3 mb-6">
              <Heart className="h-6 w-6 text-gold-light" />
              <h2 className="font-serif text-2xl font-bold">Favorite Songs</h2>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {[1, 2, 3, 4].map((i) => (
                <div
                  key={i}
                  className="flex items-center gap-4 p-4 rounded-xl bg-muted/20 hover:bg-muted/40 transition-colors cursor-pointer"
                >
                  <div className="w-12 h-12 rounded bg-gradient-to-br from-gold to-gold-light flex items-center justify-center flex-shrink-0">
                    <Music2 className="h-6 w-6 text-background" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold truncate">Favorite Song {i}</h3>
                    <p className="text-sm text-muted-foreground truncate">Artist</p>
                  </div>
                </div>
              ))}
            </div>
          </GlassCard>
        </div>
      </div>
    </div>
  );
};

export default Profile;
