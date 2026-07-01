import DefaultTheme from 'vitepress/theme';
import { nextTick, onMounted, watch } from 'vue';
import { useRoute } from 'vitepress';
import mediumZoom from 'medium-zoom';
import './custom.css';

function enableImageZoom() {
  const images = Array.from(
    document.querySelectorAll<HTMLImageElement>('.vp-doc img')
  ).filter((image) => !image.closest('.mermaid'));
  if (images.length > 0) {
    mediumZoom(images, {
      background: 'rgba(12, 17, 23, 0.92)',
      margin: 24
    });
  }
}

function enableMermaidZoom() {
  document.querySelectorAll<HTMLElement>('.mermaid').forEach((node) => {
    if (node.dataset.zoomReady === 'true') {
      return;
    }
    node.dataset.zoomReady = 'true';
    node.style.cursor = 'zoom-in';
    node.addEventListener('click', () => {
      const modal = document.createElement('div');
      modal.className = 'mermaid-zoom-modal';
      const clone = node.cloneNode(true) as HTMLElement;
      clone.removeAttribute('data-zoom-ready');
      modal.appendChild(clone);
      modal.addEventListener('click', () => modal.remove());
      document.body.appendChild(modal);
    });
  });
}

function runEnhancements() {
  nextTick(() => {
    enableImageZoom();
    enableMermaidZoom();
  });
}

export default {
  extends: DefaultTheme,
  setup() {
    const route = useRoute();
    onMounted(() => {
      runEnhancements();
      watch(
        () => route.path,
        () => runEnhancements()
      );
    });
  }
};
