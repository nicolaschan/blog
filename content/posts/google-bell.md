---
title: "Google Chrome Blocks Entire Domain over NextAuth Defaults"
date: 2023-07-29T22:05:43-07:00
tags: ["nextauth", "bell"]
---

# Summary

> **Update August 2, 2023**: Google sent an email yesterday August 1, 2023 saying that they have processed the security review request I had submitted on July 29, 2023. The email says that "bell.plus no longer contains links to harmful sites or downloads. The warnings visible to users are being removed from your site." While I'm glad that the warnings are being removed, I am still worried that the issue described below persists. I had removed the NextAuth default login page and replaced it with my own. It's unclear to me whether this was a manual review or if their scan no longer found that page.

Google is falsely flagging [NextAuth](https://next-auth.js.org/)'s default sign in page as phishing. NextAuth is a popular auth library for Next.js. This will get your entire domain, including all subdomains, blocked by Google Chrome for "phishing." Here is my recommendation:

- If you use NextAuth on your website, immediately stop using the default sign in page.
- Help [report an incorrect phishing warning](https://safebrowsing.google.com/safebrowsing/report_error/?url=https%3A%2F%2Fedit.bell.plus%2Fapi%2Fauth%2Fsignin%3FcallbackUrl%3D%2Fschools&hl=en-US).
- Spread the word! Google needs to patch this before more innocent sites are banned.
- Consider using a browser other than Google Chrome or Firefox (both use the Google block list).

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
