# 🎌 Sonica Frontend - Lovable Anime Style Prompt

## 🎯 Project Overview

Build **Sonica** - an ultra-fast music recognition app with an **anime-inspired UI/UX design**. The app should feel like a modern anime interface with smooth animations, vibrant colors, and Japanese aesthetic elements.

## 🎨 Anime Style Design Requirements

### **Color Palette**
```css
/* Primary Colors - Anime Inspired */
--primary-purple: #8B5CF6;      /* Vibrant purple like anime magic */
--primary-pink: #EC4899;        /* Hot pink for accents */
--primary-blue: #3B82F6;        /* Electric blue */
--primary-cyan: #06B6D4;        /* Cyan for highlights */

/* Secondary Colors */
--secondary-orange: #F97316;    /* Orange for energy */
--secondary-yellow: #EAB308;    /* Golden yellow */
--secondary-green: #10B981;     /* Emerald green */

/* Background Colors */
--bg-dark: #0F0F23;            /* Deep space blue */
--bg-card: #1A1A2E;            /* Dark card background */
--bg-glass: rgba(255, 255, 255, 0.1); /* Glass morphism */

/* Text Colors */
--text-primary: #FFFFFF;        /* Pure white */
--text-secondary: #A1A1AA;      /* Light gray */
--text-accent: #FBBF24;         /* Gold accent text */
```

### **Typography - Anime Style**
```css
/* Headers - Bold and Dynamic */
font-family: 'Inter', 'Noto Sans JP', sans-serif;
font-weight: 700-900;
text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);

/* Body Text */
font-family: 'Inter', 'Noto Sans JP', sans-serif;
font-weight: 400-600;

/* Special Effects */
text-shadow: 0 0 10px rgba(139, 92, 246, 0.5); /* Glowing text */
```

### **Visual Elements**
- **Gradient Backgrounds**: Purple to pink gradients
- **Glass Morphism**: Frosted glass cards with blur effects
- **Neon Glows**: Subtle glowing borders and shadows
- **Particle Effects**: Floating particles in background
- **Anime Icons**: Custom SVG icons with anime style
- **Smooth Animations**: 60fps transitions and micro-interactions

## 🏗️ Technical Requirements

### **Framework & Tools**
- **Next.js 14** with App Router
- **TypeScript** for type safety
- **Tailwind CSS** for styling
- **Framer Motion** for animations
- **React Hook Form** for forms
- **Zustand** for state management

### **Core Features to Build**

#### 1. **Audio Recognition Interface**
```tsx
// Main recognition component with anime styling
const AudioRecorder = () => {
  return (
    <div className="anime-recorder-container">
      {/* Animated microphone button */}
      {/* Real-time audio visualization */}
      {/* Recognition progress with anime-style loading */}
    </div>
  );
};
```

#### 2. **Song Results Display**
```tsx
// Results with anime card design
const SongResult = ({ song }) => {
  return (
    <div className="anime-song-card">
      {/* Album artwork with hover effects */}
      {/* Song info with glowing text */}
      {/* Action buttons with anime styling */}
    </div>
  );
};
```

#### 3. **User Dashboard**
```tsx
// Dashboard with anime dashboard aesthetic
const Dashboard = () => {
  return (
    <div className="anime-dashboard">
      {/* Stats cards with glass morphism */}
      {/* Recognition history with anime list items */}
      {/* Favorites with heart animations */}
    </div>
  );
};
```

## 🎌 Anime UI Components

### **1. Animated Buttons**
```tsx
// Primary action button with anime effects
<button className="anime-btn-primary">
  <span className="btn-glow"></span>
  <span className="btn-text">Recognize Music</span>
  <span className="btn-particles"></span>
</button>

// Secondary button with hover effects
<button className="anime-btn-secondary">
  <span className="btn-ripple"></span>
  <span className="btn-text">Search Songs</span>
</button>
```

### **2. Glass Morphism Cards**
```tsx
// Song result card with glass effect
<div className="anime-glass-card">
  <div className="card-glow"></div>
  <div className="card-content">
    {/* Card content */}
  </div>
</div>
```

### **3. Loading Animations**
```tsx
// Anime-style loading spinner
<div className="anime-loader">
  <div className="loader-ring"></div>
  <div className="loader-particles"></div>
  <span className="loader-text">Recognizing...</span>
</div>
```

### **4. Audio Visualizer**
```tsx
// Real-time audio visualization
<div className="anime-audio-visualizer">
  <div className="visualizer-bars">
    {bars.map((bar, index) => (
      <div 
        key={index}
        className="visualizer-bar"
        style={{ height: `${bar}%` }}
      />
    ))}
  </div>
</div>
```

## 🎵 Core Pages to Build

### **1. Landing Page**
```tsx
const LandingPage = () => {
  return (
    <div className="anime-landing">
      {/* Hero section with animated background */}
      {/* Feature showcase with anime cards */}
      {/* CTA section with glowing buttons */}
    </div>
  );
};
```

### **2. Recognition Page**
```tsx
const RecognitionPage = () => {
  return (
    <div className="anime-recognition">
      {/* Audio recorder with anime styling */}
      {/* Real-time feedback */}
      {/* Results display */}
    </div>
  );
};
```

### **3. Search Page**
```tsx
const SearchPage = () => {
  return (
    <div className="anime-search">
      {/* Search bar with anime styling */}
      {/* Filter options */}
      {/* Results grid */}
    </div>
  );
};
```

### **4. User Profile**
```tsx
const ProfilePage = () => {
  return (
    <div className="anime-profile">
      {/* User avatar with anime styling */}
      {/* Stats dashboard */}
      {/* Recognition history */}
      {/* Favorites list */}
    </div>
  );
};
```

## 🎨 Animation Specifications

### **Page Transitions**
```tsx
// Smooth page transitions with anime feel
const pageVariants = {
  initial: { opacity: 0, y: 20 },
  in: { opacity: 1, y: 0 },
  out: { opacity: 0, y: -20 }
};

const pageTransition = {
  type: "tween",
  ease: "anticipate",
  duration: 0.5
};
```

### **Micro-interactions**
- **Button Hover**: Scale + glow effect
- **Card Hover**: Lift + shadow increase
- **Input Focus**: Border glow + label animation
- **Loading States**: Pulsing + particle effects
- **Success States**: Confetti + celebration animation

### **Audio Visualizations**
- **Recording**: Pulsing microphone with sound waves
- **Processing**: Spinning loader with music notes
- **Results**: Smooth slide-in with bounce effect

## 🎌 Anime-Specific Elements

### **1. Particle Background**
```tsx
// Floating particles like anime magic
<div className="anime-particles">
  {particles.map((particle, index) => (
    <div 
      key={index}
      className="particle"
      style={{
        left: particle.x,
        top: particle.y,
        animationDelay: particle.delay
      }}
    />
  ))}
</div>
```

### **2. Glowing Effects**
```css
/* Glowing borders and shadows */
.glow-effect {
  box-shadow: 
    0 0 20px rgba(139, 92, 246, 0.5),
    0 0 40px rgba(139, 92, 246, 0.3),
    0 0 60px rgba(139, 92, 246, 0.1);
}

.text-glow {
  text-shadow: 0 0 10px rgba(139, 92, 246, 0.8);
}
```

### **3. Gradient Overlays**
```css
/* Anime-style gradients */
.anime-gradient {
  background: linear-gradient(
    135deg,
    #8B5CF6 0%,
    #EC4899 50%,
    #3B82F6 100%
  );
}
```

## 📱 Responsive Design

### **Mobile-First Approach**
- **Touch-friendly** buttons (44px minimum)
- **Swipe gestures** for navigation
- **Optimized layouts** for small screens
- **Fast loading** with progressive enhancement

### **Breakpoints**
```css
/* Tailwind breakpoints */
sm: 640px   /* Mobile landscape */
md: 768px   /* Tablet */
lg: 1024px  /* Desktop */
xl: 1280px  /* Large desktop */
2xl: 1536px /* Extra large */
```

## 🎵 Audio Integration

### **Web Audio API**
```tsx
// Audio recording with anime visualizations
const useAudioRecorder = () => {
  const [isRecording, setIsRecording] = useState(false);
  const [audioData, setAudioData] = useState(null);
  
  const startRecording = async () => {
    // Web Audio API implementation
    // Real-time visualization
    // Anime-style feedback
  };
  
  return { isRecording, audioData, startRecording };
};
```

### **Audio Visualization**
- **Real-time bars** that dance to music
- **Particle effects** that respond to audio
- **Color changes** based on frequency
- **Smooth animations** for all visual elements

## 🚀 Performance Requirements

### **Optimization**
- **Code splitting** for fast loading
- **Image optimization** with Next.js
- **Lazy loading** for components
- **Service workers** for offline capability

### **Animations**
- **60fps** smooth animations
- **GPU acceleration** for transforms
- **Reduced motion** support
- **Performance monitoring**

## 🎌 Specific Anime References

### **Design Inspiration**
- **Studio Ghibli** color palettes
- **Makoto Shinkai** gradient backgrounds
- **Kyoto Animation** smooth animations
- **Ufotable** particle effects
- **Bones** dynamic layouts

### **UI Elements**
- **Floating cards** like in anime interfaces
- **Glowing buttons** with energy effects
- **Smooth transitions** between states
- **Particle systems** for magic effects
- **Gradient overlays** for depth

## 📋 Implementation Checklist

### **Phase 1: Core Components**
- [ ] Audio recorder with anime styling
- [ ] Song result cards with glass morphism
- [ ] Loading animations with particles
- [ ] Navigation with smooth transitions

### **Phase 2: Pages**
- [ ] Landing page with hero section
- [ ] Recognition page with visualizer
- [ ] Search page with filters
- [ ] Profile page with dashboard

### **Phase 3: Polish**
- [ ] Micro-interactions and hover effects
- [ ] Particle background system
- [ ] Responsive design optimization
- [ ] Performance optimization

## 🎯 Success Criteria

### **Visual Appeal**
- **Anime aesthetic** that feels authentic
- **Smooth animations** at 60fps
- **Consistent design** language
- **Mobile-optimized** experience

### **Functionality**
- **Audio recording** works seamlessly
- **Real-time feedback** during recognition
- **Fast loading** and smooth navigation
- **Offline capability** with service workers

### **User Experience**
- **Intuitive interface** for music recognition
- **Engaging animations** that enhance UX
- **Accessible design** with proper contrast
- **Performance** that feels instant

## 🎌 Final Notes

Create a **magical, anime-inspired music recognition app** that feels like using a futuristic interface from your favorite anime. The app should be:

- **Visually stunning** with anime aesthetics
- **Functionally powerful** for music recognition
- **Smoothly animated** with 60fps performance
- **Mobile-first** with touch-friendly interactions
- **Accessible** with proper contrast and navigation

Make it feel like **Sonica** is the music recognition app that would exist in an anime universe - fast, beautiful, and magical! ✨🎵
