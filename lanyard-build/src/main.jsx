import React from 'react';
import { createRoot } from 'react-dom/client';
import TeamLanyardRail from './TeamLanyardRail.tsx';
import './lanyard.css';
const items = [
 {id:'alku',name:'ALKÜ Hackathon',role:'2025 · Hackathon',team:'YARIŞMA',image:'/assets/ai_post_ecosystem.png',tone:1},
 {id:'fibonacci',name:'Fibonacci Robot Olimpiyatları',role:'2025 · Robotik',team:'YARIŞMA',image:'/assets/sumo_robot.png',tone:2},
 {id:'meb',name:'MEB Robot',role:'2026 · Robotik',team:'YARIŞMA',image:'/assets/nexusai_phone.png',tone:3}
];
createRoot(document.getElementById('lanyard-root')).render(<TeamLanyardRail items={items} caption="Kartları çek, bırak ve çevirmek için dokun." labels={{flip:'Kartı çevirmek için dokun',back:'Geri çevirmek için dokun',show:'Göster'}} />);
