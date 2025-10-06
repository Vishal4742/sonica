export const ParticleBackground = () => {
  // Minimal particles for luxury aesthetic
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden opacity-20">
      <div className="absolute top-1/4 left-1/4 w-1 h-1 bg-gold rounded-full animate-pulse" />
      <div className="absolute top-2/3 right-1/3 w-1 h-1 bg-gold rounded-full animate-pulse" style={{ animationDelay: "1s" }} />
      <div className="absolute bottom-1/4 left-2/3 w-1 h-1 bg-gold rounded-full animate-pulse" style={{ animationDelay: "2s" }} />
    </div>
  );
};
