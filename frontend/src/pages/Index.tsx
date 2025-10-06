import { Link } from "react-router-dom";
import { ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import luxuryHero from "@/assets/luxury-hero.jpg";

const Index = () => {
  const currentDate = new Date().toLocaleDateString("en-US", {
    month: "long",
    day: "2-digit",
    year: "numeric",
  }).toUpperCase();

  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* Geometric background elements */}
      <div className="absolute inset-0 opacity-5">
        <div className="absolute top-20 right-1/4 text-9xl font-serif text-gold">0°</div>
        <div className="absolute bottom-1/3 left-1/3 text-9xl font-serif text-gold">0°</div>
        <div className="absolute top-1/2 right-1/3 text-9xl font-serif text-gold">0°</div>
      </div>

      {/* Diagonal lines */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/4 w-px h-full bg-gradient-to-b from-transparent via-border/30 to-transparent transform -rotate-12" />
        <div className="absolute top-0 right-1/3 w-px h-full bg-gradient-to-b from-transparent via-border/30 to-transparent transform rotate-12" />
      </div>

      {/* Sidebar Navigation */}
      <div className="fixed left-8 top-1/2 -translate-y-1/2 hidden lg:flex flex-col gap-8 z-10">
        {[1, 2, 3, 4].map((num) => (
          <div
            key={num}
            className={cn(
              "text-2xl font-light transition-colors cursor-pointer",
              num === 1 ? "text-gold" : "text-muted-foreground hover:text-gold"
            )}
          >
            {num}
          </div>
        ))}
      </div>

      <div className="container mx-auto px-6 lg:px-12 pt-32 pb-20">
        <div className="grid lg:grid-cols-2 gap-12 items-center min-h-[calc(100vh-8rem)]">
          {/* Left Content */}
          <div className="space-y-8 animate-fade-in">
            {/* Signature Title */}
            <div className="space-y-6">
              <h1 className="font-serif text-7xl lg:text-8xl font-bold text-gold italic leading-none">
                Sonica
              </h1>
              <p className="text-sm text-muted-foreground max-w-md leading-relaxed">
                Developed with cutting-edge AI technology for high-precision
                audio recognition and instant music identification.
              </p>
            </div>

            {/* CTA Button */}
            <div className="pt-4">
              <Link to="/recognize">
                <Button variant="luxury" size="lg" className="group">
                  Recognize
                  <ArrowRight className="h-4 w-4 group-hover:translate-x-1 transition-transform" />
                </Button>
              </Link>
            </div>

            {/* Date */}
            <div className="pt-12">
              <p className="text-xs tracking-widest text-muted-foreground">
                {currentDate}
              </p>
            </div>

            {/* Main Heading */}
            <div className="pt-8 space-y-4 max-w-lg">
              <h2 className="font-serif text-4xl lg:text-5xl leading-tight text-gold-light">
                Make Music Discovery Your Best Digital Experience
              </h2>
              <p className="text-sm text-muted-foreground leading-relaxed">
                Developed with advanced AI algorithms for instant
                audio fingerprinting and real-time music identification.
              </p>
            </div>
          </div>

          {/* Right Image */}
          <div className="relative animate-fade-in-up lg:ml-auto">
            <div className="relative w-full max-w-md lg:max-w-lg mx-auto">
              <img
                src={luxuryHero}
                alt="Luxury music experience"
                className="w-full h-auto rounded-sm"
              />
              
              {/* Explore Label */}
              <div className="absolute bottom-8 -right-4 lg:-right-12">
                <div className="flex items-center gap-4">
                  <div className="h-24 w-px bg-gold" />
                  <div className="writing-mode-vertical text-sm tracking-widest uppercase text-gold font-medium">
                    Explore
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Stats */}
      <div className="fixed bottom-8 left-1/2 -translate-x-1/2 hidden lg:flex items-center gap-12 text-xs tracking-widest text-muted-foreground">
        <div>
          <span className="text-gold font-medium">10M+</span> RECOGNITIONS
        </div>
        <div className="w-px h-4 bg-border" />
        <div>
          <span className="text-gold font-medium">99%</span> ACCURACY
        </div>
        <div className="w-px h-4 bg-border" />
        <div>
          <span className="text-gold font-medium">&lt;3s</span> SPEED
        </div>
      </div>
    </div>
  );
};

// Helper for cn (added here since it's needed)
import { cn } from "@/lib/utils";

export default Index;
