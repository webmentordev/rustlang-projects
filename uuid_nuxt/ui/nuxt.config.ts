import tailwindcss from "@tailwindcss/vite";
// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  css: ['./app/assets/css/tailwind.css'],
  vite: {
    plugins: [
      tailwindcss(),
    ],
  },
  app: {
    head: {
      title: 'Generate Unlimited V4 UUID Online with File | Free UUID Generator',
      htmlAttrs: {
        lang: 'en',
      },
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        { 
          name: 'title', 
          content: 'Generate Unlimited V4 UUID Online with File | Free UUID Generator' 
        },
        { 
          name: 'description', 
          content: 'Free online UUID v4 generator. Create unlimited unique identifiers instantly. Generate single or bulk UUIDs, download as file, copy to clipboard. No registration required. Fast, secure, and 100% free.' 
        },
        { 
          name: 'keywords', 
          content: 'UUID generator, UUID v4, generate UUID, random UUID, unique identifier, GUID generator, bulk UUID, UUID online, free UUID generator, download UUID file' 
        },
        { name: 'robots', content: 'index, follow' },
        { name: 'author', content: 'Your Name or Company' },
        { name: 'language', content: 'English' },
        { property: 'og:type', content: 'website' },
        { property: 'og:url', content: 'https://yourdomain.com/' },
        { 
          property: 'og:title', 
          content: 'Generate Unlimited V4 UUID Online with File | Free UUID Generator' 
        },
        { 
          property: 'og:description', 
          content: 'Free online UUID v4 generator. Create unlimited unique identifiers instantly. Generate single or bulk UUIDs, download as file, copy to clipboard.' 
        },
        { property: 'og:image', content: 'https://yourdomain.com/og-image.png' },
        { property: 'og:image:width', content: '1200' },
        { property: 'og:image:height', content: '630' },
        { property: 'og:site_name', content: 'UUID Generator' },
        { name: 'twitter:card', content: 'summary_large_image' },
        { name: 'twitter:url', content: 'https://yourdomain.com/' },
        { 
          name: 'twitter:title', 
          content: 'Generate Unlimited V4 UUID Online with File | Free UUID Generator' 
        },
        { 
          name: 'twitter:description', 
          content: 'Free online UUID v4 generator. Create unlimited unique identifiers instantly. Generate single or bulk UUIDs, download as file, copy to clipboard.' 
        },
        { name: 'twitter:image', content: 'https://yourdomain.com/twitter-image.png' },
        { name: 'theme-color', content: '#ffffff' },
        { name: 'format-detection', content: 'telephone=no' },
        { name: 'apple-mobile-web-app-capable', content: 'yes' },
        { name: 'apple-mobile-web-app-status-bar-style', content: 'default' },
      ],
      link: [
        { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
        { rel: 'icon', type: 'image/png', sizes: '32x32', href: '/favicon-32x32.png' },
        { rel: 'icon', type: 'image/png', sizes: '16x16', href: '/favicon-16x16.png' },
        { rel: 'apple-touch-icon', sizes: '180x180', href: '/apple-touch-icon.png' },
        { rel: 'manifest', href: '/site.webmanifest' },
        { rel: 'canonical', href: 'https://yourdomain.com/' },
      ],
    },
  }
})
