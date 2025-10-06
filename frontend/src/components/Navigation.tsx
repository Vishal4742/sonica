import { Link, useLocation } from "react-router-dom";
import { Music2, Menu } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState } from "react";

export const Navigation = () => {
  const location = useLocation();
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const links = [
    { to: "/", label: "Home" },
    { to: "/recognize", label: "Recognize" },
    { to: "/search", label: "Search" },
    { to: "/profile", label: "Profile" },
  ];

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-background/80 backdrop-blur-md border-b border-border/30">
      <div className="container mx-auto px-6 lg:px-12">
        <div className="flex items-center justify-between h-20">
          {/* Menu Icon - Left */}
          <button 
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            className="hover:text-gold transition-colors"
          >
            <Menu className="h-6 w-6" />
          </button>

          {/* Logo - Center */}
          <Link to="/" className="absolute left-1/2 -translate-x-1/2 flex items-center gap-3">
            <Music2 className="h-7 w-7 text-gold" />
            <span className="font-serif text-3xl font-bold text-gold tracking-wide italic">
              Sonica
            </span>
          </Link>

          {/* Right Links */}
          <div className="hidden md:flex items-center gap-8">
            {links.slice(0, 2).map(({ to, label }) => {
              const isActive = location.pathname === to;
              return (
                <Link
                  key={to}
                  to={to}
                  className={cn(
                    "text-sm uppercase tracking-widest transition-colors relative",
                    isActive
                      ? "text-gold font-medium"
                      : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  {label}
                  {isActive && (
                    <span className="absolute -bottom-1 left-0 w-full h-px bg-gold" />
                  )}
                </Link>
              );
            })}
          </div>
        </div>
      </div>

      {/* Mobile Menu */}
      {isMenuOpen && (
        <div className="md:hidden absolute top-20 left-0 right-0 bg-card/95 backdrop-blur-lg border-b border-border/30 animate-fade-in">
          <div className="container mx-auto px-6 py-6 space-y-4">
            {links.map(({ to, label }) => (
              <Link
                key={to}
                to={to}
                onClick={() => setIsMenuOpen(false)}
                className={cn(
                  "block text-sm uppercase tracking-widest transition-colors",
                  location.pathname === to
                    ? "text-gold font-medium"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                {label}
              </Link>
            ))}
          </div>
        </div>
      )}
    </nav>
  );
};
