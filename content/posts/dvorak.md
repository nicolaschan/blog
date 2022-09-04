---
title: "Dvorak"
date: 2022-09-03T23:08:48-07:00
tags: ["typing"]
---

I started using [the Dvorak keyboard layout](https://en.wikipedia.org/wiki/Dvorak_keyboard_layout) in middle school. Here are my thoughts as a full-time Dvorak typist ten years later.

## Observations

- I'm not much faster than my peak Qwerty speed, if at all (&asymp;110 WPM). This isn't super fast, but my typing speed usually exceeds my thinking speed so it's enough for me.

- My Qwerty speed has degraded since peak, but not terribly. My Qwerty speed is around 70 WPM. This is good considering how rarely I use it.

- I love Vim keybindings, but my muscle memory is for _Dvorak_ Vim bindings. In Qwerty, the Vim movement keys are all together: `HJKL`. In Dvorak, this would be the on the physical Qwerty keys labeled `JCVP`. By pure coincidence, this works out to be fine since the `C-V` keys are Down-Up and `J-P` are Left-Right. It's not how Vim was intended but it works well enough for me.

- I'm a touch typist so there is no need to relabel the keys. After typing a lot of $\LaTeX$, I have memorized the locations of most special characters.

- I recently switched to using caps lock as control. This is definitely an improvement for combos like `Ctrl-A`.

## Pros

- I never get finger fatigue, even after typing for long periods. When comparing with Qwerty I _definitely_ notice that my fingers travel a lot less in Dvorak.

- While Dvorak is never the default, it is widely supported. Every Linux distro I've tried, Windows, and Mac all have it out of the box with no special software needed. This is a reason you might choose Dvorak over Colemak (until Colemak makes it into Windows).

- Dvorak acts as deterrence if I forgot to lock my computer and someone tries to mess with it while I'm away. &#x1F608;

## Cons

- Dvorak moves keyboard shortcuts. Some people remap the shortcuts to keep them in the same place they would be physically on Qwerty[^1]. I can't be bothered, so I use the Dvorak equivalents. It is less convenient not having copy and paste next to each other, but I'm used to it now.

- It's a lonely life. Occasionally there is the subtle reminder that it's a Qwerty-typist's world. Some common terminal commands like `ls` are easier to type in Qwerty than Dvorak. Some games _incorrectly_ check the character instead of the physical key[^2]. In those cases, I have to manually rebind the typical `WASD` movement keys to Dvorak's `,AOE`.

## Conclusion

Do the pros outweigh the cons? Just barely. The pros are not very strong but neither are the cons. Dvorak feels more ergonomic, so I wouldn't switch back to Qwerty.

If I were to have to make the choice again, I'd still choose Dvorak to be special. &#x1F643;

[^1]: On Mac, this is "Dvorak - Qwerty &#x2318;"
[^2]: This is also an internationalization issue. Use something like [key code](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code) if you rely on physical key layout.