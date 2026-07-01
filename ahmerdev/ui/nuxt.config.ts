// https://nuxt.com/docs/api/configuration/nuxt-config
import tailwindcss from "@tailwindcss/vite";

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  css: ['./app/assets/css/main.css'],
  ssr: false,
  vite: {
    plugins: [
      tailwindcss(),
    ],
  },
  app: {
    head: {
      title: 'Muhammad Ahmer Tahir - Software Engineer | Laravel, PHP, Livewire & Rust Developer',
      charset: 'utf-8',
      viewport: 'width=device-width, initial-scale=1, maximum-scale=1',
      htmlAttrs: {
        lang: 'en',
      },
      meta: [
        {
          name: 'description',
          content: 'Muhammad Ahmer Tahir - Full Stack Software Engineer specializing in Laravel, PHP, Livewire, and Rust. Experienced developer passionate about building scalable web applications.',
        },
        {
          name: 'keywords',
          content: 'Muhammad Ahmer Tahir, Laravel developer, PHP engineer, Livewire, Rust, full stack developer, web development',
        },
        {
          name: 'author',
          content: 'Muhammad Ahmer Tahir',
        },
        {
          name: 'robots',
          content: 'index, follow',
        },
        {
          property: 'og:type',
          content: 'profile',
        },
        {
          property: 'og:title',
          content: 'Muhammad Ahmer Tahir - Software Engineer',
        },
        {
          property: 'og:description',
          content: 'Full Stack Software Engineer specializing in Laravel, PHP, Livewire, and Rust.',
        },
        {
          property: 'og:url',
          content: 'https://ahmerdev.online',
        },
        {
          property: 'og:image',
          content: '/assets/favicon.jpeg',
        },
        {
          property: 'og:site_name',
          content: 'Muhammad Ahmer Tahir Portfolio',
        },
        {
          name: 'twitter:card',
          content: 'summary_large_image',
        },
        {
          name: 'twitter:title',
          content: 'Muhammad Ahmer Tahir - Software Engineer',
        },
        {
          name: 'twitter:description',
          content: 'Laravel, PHP, Livewire & Rust Developer | Full Stack Engineer',
        },
        {
          name: 'twitter:image',
          content: '/assets/favicon.jpeg',
        },
        {
          name: 'theme-color',
          content: '#ffffff',
        }
      ],
      link: [
        { rel: 'icon', type: 'image/x-icon', href: '/assets/favicon.png' },
        { rel: 'canonical', href: 'https://yourdomain.com/profile' },
      ],
    },
  },
})
