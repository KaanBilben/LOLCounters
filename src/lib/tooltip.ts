import type { Action } from 'svelte/action';

// One shared tooltip element for the whole app. It is only positioned on
// hover / keyboard focus; no timers or listeners run while nothing is shown.

let tip: HTMLDivElement | null = null;
let owner: HTMLElement | null = null;

function element(): HTMLDivElement {
  if (!tip) {
    tip = document.createElement('div');
    tip.className = 'app-tooltip';
    tip.setAttribute('role', 'tooltip');
    tip.id = 'app-tooltip';
    document.body.appendChild(tip);
  }
  return tip;
}

function onScroll() {
  hide();
}

function show(node: HTMLElement, text: string) {
  const el = element();
  el.textContent = text;
  owner = node;
  const r = node.getBoundingClientRect();
  const t = el.getBoundingClientRect();
  const gap = 8;
  let top = r.top - t.height - gap;
  let below = false;
  if (top < 4) {
    top = r.bottom + gap;
    below = true;
  }
  const left = Math.min(
    Math.max(6, r.left + r.width / 2 - t.width / 2),
    window.innerWidth - t.width - 6
  );
  el.style.left = `${Math.round(left)}px`;
  el.style.top = `${Math.round(top)}px`;
  el.classList.toggle('below', below);
  el.classList.add('visible');
  node.setAttribute('aria-describedby', 'app-tooltip');
  window.addEventListener('scroll', onScroll, { capture: true, passive: true });
}

function hide(node?: HTMLElement) {
  if (!tip || !owner || (node && node !== owner)) return;
  owner.removeAttribute('aria-describedby');
  tip.classList.remove('visible');
  owner = null;
  window.removeEventListener('scroll', onScroll, { capture: true });
}

/** `use:tooltip={'Text'}` — custom tooltip on hover and keyboard focus. */
export const tooltip: Action<HTMLElement, string | null | undefined> = (node, initial) => {
  let text = initial;
  const enter = () => text && show(node, text);
  const focus = () => {
    if (text && node.matches(':focus-visible')) show(node, text);
  };
  const leave = () => hide(node);
  const key = (e: KeyboardEvent) => {
    if (e.key === 'Escape') hide(node);
  };
  node.addEventListener('mouseenter', enter);
  node.addEventListener('mouseleave', leave);
  node.addEventListener('focus', focus);
  node.addEventListener('blur', leave);
  node.addEventListener('pointerdown', leave);
  node.addEventListener('keydown', key);
  return {
    update(next) {
      text = next;
      if (owner === node) {
        if (next) show(node, next);
        else hide(node);
      }
    },
    destroy() {
      node.removeEventListener('mouseenter', enter);
      node.removeEventListener('mouseleave', leave);
      node.removeEventListener('focus', focus);
      node.removeEventListener('blur', leave);
      node.removeEventListener('pointerdown', leave);
      node.removeEventListener('keydown', key);
      hide(node);
    }
  };
};
