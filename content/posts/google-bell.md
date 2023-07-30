---
title: "Google Chrome Blocks Entire Domain over NextAuth Defaults"
date: 2023-07-29T22:05:43-07:00
tags: ["nextauth", "bell"]
---

# Summary

Google is falsely flagging [NextAuth](https://next-auth.js.org/)'s default sign in page as phishing. NextAuth is a popular auth library for Next.js. This will get your entire domain, including all subdomains, blocked by Google Chrome for "phishing." Here is my recommendation:

- If you use NextAuth on your website, immediately stop using the default sign in page.
- Help [report an incorrect phishing warning](https://safebrowsing.google.com/safebrowsing/report_error/?url=https%3A%2F%2Fedit.bell.plus%2Fapi%2Fauth%2Fsignin%3FcallbackUrl%3D%2Fschools&hl=en-US).
- Spread the word! Google needs to patch this before more innocent sites are banned.
- Consider using another browser other than Google Chrome or Firefox, which also uses the Google block list.

# What happened

Today Google Chrome blocked my domain `bell.plus` and all of its subdomains for "phishing" due to the NextAuth default sign in page. This is a website I run for free to help students with their schedules. Signing in is optional and only needed if you want to share a schedule.

According to the Google Search console, the "phishing" page is the default NextAuth sign in page: 

[![Google Search Console report](/static/img/search-console.png)](/static/img/search-console.png)

It is simply a list of buttons to use to sign in with a provider. There is no deception because when a user clicks a provider they are taken to the provider's website and asked to consent to sharing their identity with this third party. In fact, this sign in page appears on `edit.<domain>`, but _all_ subdomains are blocked by Google Chrome.

[![NextAuth default sign in page](/static/img/nextauth.png)](/static/img/nextauth.png)

A quick search shows that this is a recurring pattern with the NextAuth default sign in page:
- [Google reports NextAuth api page site as phishing/social engineering](https://github.com/nextauthjs/next-auth/discussions/7465)
- [Site marked as deceptive for Phishing, even when it is not](https://stackoverflow.com/questions/75698532/site-marked-as-deceptive-for-phishing-even-when-it-is-not)
- [My website has been marked as dangerous by Google Chrome](https://stackoverflow.com/questions/75599960/my-website-has-been-marked-as-dangerous-by-google-chrome?rq=2)

There are likely many more instances of this false-positive that haven't spoken out publicly. While I hope my own issue is resolved, I write this post to warn others and hopefully spread the word to fix this issue. Falsely flagged websites are bad not only for the website and its users but also because it encourages users to normalize bypassing the security warnings.
