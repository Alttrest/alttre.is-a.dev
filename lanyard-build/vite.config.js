import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
export default defineConfig({define:{'process.env.NODE_ENV':JSON.stringify('production')},plugins:[tailwindcss()],build:{outDir:'../lanyard-dist',emptyOutDir:true,lib:{entry:'src/main.jsx',formats:['iife'],name:'LanyardApp',fileName:()=> 'lanyard.js',cssFileName:'lanyard'},cssCodeSplit:false}});
