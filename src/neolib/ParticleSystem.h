#ifndef __NEOPARTICLESYSTEM_H
#define __NEOPARTICLESYSTEM_H

#include "Renderer.h"

class Particle {
public:
  Particle(float x1, float y1, Color c1);
  void Update(std::vector<Particle> &particleArr);
  void Draw(Renderer &renderer);

private:
  float x, y;
  float alpha = 255;
  float vx, vy;
  float ax;
  float ay;
  Color c;
};



class ParticleSystem
{
private:
    std::vector<Particle> particleArr;
public:
    ParticleSystem();
    void UpdateParticles();
    void RemoveParticles();
    void DrawParticles(Renderer &renderer);
    void SpawnParticle(float x1, float y1, Color c1);
};



#endif