import { useEffect, useState, useRef } from "react";
import { cn } from "@/lib/utils";
import { audioProcessor } from "@/lib/audioProcessor";

interface AudioVisualizerProps {
  isActive?: boolean;
  barCount?: number;
  className?: string;
  showParticles?: boolean;
}

export const AudioVisualizer = ({ 
  isActive = false, 
  barCount = 40,
  className,
  showParticles = true
}: AudioVisualizerProps) => {
  const [bars, setBars] = useState<number[]>([]);
  const [particles, setParticles] = useState<Array<{ id: number; x: number; y: number; life: number }>>([]);
  const animationRef = useRef<number>();
  const particleIdRef = useRef(0);

  useEffect(() => {
    if (isActive) {
      const animate = () => {
        // Get real audio data if available
        const audioData = audioProcessor.getAudioData();
        
        if (audioData) {
          // Process real audio data
          const newBars = Array.from({ length: barCount }, (_, i) => {
            const dataIndex = Math.floor((i / barCount) * audioData.length);
            return (audioData[dataIndex] / 255) * 100;
          });
          setBars(newBars);

          // Generate particles based on audio intensity
          if (showParticles) {
            const avgIntensity = audioData.reduce((sum, val) => sum + val, 0) / audioData.length;
            if (avgIntensity > 50) {
              const newParticle = {
                id: particleIdRef.current++,
                x: Math.random() * 100,
                y: 100,
                life: 1.0
              };
              setParticles(prev => [...prev.slice(-20), newParticle]);
            }
          }
        } else {
          // Fallback to random data for demo
          setBars(Array.from({ length: barCount }, () => Math.random() * 100));
        }

        // Update particle positions
        if (showParticles) {
          setParticles(prev => 
            prev
              .map(p => ({ ...p, y: p.y - 2, life: p.life - 0.02 }))
              .filter(p => p.life > 0)
          );
        }

        animationRef.current = requestAnimationFrame(animate);
      };
      
      animationRef.current = requestAnimationFrame(animate);
    } else {
      setBars(Array.from({ length: barCount }, () => 0));
      setParticles([]);
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isActive, barCount, showParticles]);

  return (
    <div className={cn("relative flex items-end justify-center gap-1 h-32 overflow-hidden", className)}>
      {/* Audio bars */}
      {bars.map((height, index) => (
        <div
          key={index}
          className={cn(
            "w-1 rounded-full transition-all duration-75 relative",
            isActive 
              ? "bg-gradient-to-t from-gold-dark via-gold to-gold-light shadow-lg shadow-gold/30" 
              : "bg-muted"
          )}
          style={{
            height: `${isActive ? Math.max(height, 5) : 5}%`,
            animationDelay: `${index * 0.02}s`,
            boxShadow: isActive && height > 30 ? '0 0 10px hsl(var(--gold))' : 'none',
          }}
        />
      ))}

      {/* Floating particles */}
      {showParticles && particles.map((particle) => (
        <div
          key={particle.id}
          className="absolute w-1 h-1 bg-gold rounded-full opacity-60 animate-pulse"
          style={{
            left: `${particle.x}%`,
            bottom: `${particle.y}%`,
            opacity: particle.life,
            transform: `scale(${particle.life})`,
            transition: 'all 0.1s ease-out',
          }}
        />
      ))}

      {/* Glow effect overlay */}
      {isActive && (
        <div className="absolute inset-0 bg-gradient-to-t from-gold/10 via-transparent to-gold/10 rounded-lg animate-pulse-glow" />
      )}
    </div>
  );
};
