#include "ParticleSystem.h"
#include "Renderer.h"

ParticleSystem::ParticleSystem() {}

void ParticleSystem::SpawnParticle(float x1, float y1, Color c1) {
  Particle p(x1, y1, c1);
  particleArr.push_back(p);
}

void ParticleSystem::DrawParticles(Renderer &renderer) {
  glBindTexture(GL_TEXTURE_2D, 0);
  for (int i = 0; i < particleArr.size(); i++) {
    particleArr[i].Draw(renderer);
  }
}

void ParticleSystem::UpdateParticles() {
  for (int i = 0; i < particleArr.size(); i++) {
    particleArr[i].Update(particleArr);
  }
}

Particle::Particle(float x1, float y1, Color c1) : x(x1), y(y1), c(c1) {
  ax = 0.01;
  ay = -0.005;

  vx = (static_cast<float>(rand()) / static_cast<float>(RAND_MAX)) * 2 - 1;
  vy = -(static_cast<float>(rand()) / static_cast<float>(RAND_MAX)) * 2;

  c.r-=50;
  c.g-=50;
  c.b-=50;
}

void Particle::Update(std::vector<Particle> &pa) {
  vx += ax;
  vy += ay;

  x += vx;
  y += vy;

  alpha -= 1;

  if (alpha < 0) {
    if (pa.empty())
      return;
    pa.erase(pa.begin());
  }
}

void Particle::Draw(Renderer &renderer) {
  c.a = alpha;
  renderer.SetColor(c);
  renderer.DrawQuad(x, y, 2, 2);
}