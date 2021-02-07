vec3 ComputeSkyFogMie(vec3 V, vec3 L, vec3 waveLambdaMie, vec3 waveLambdaRayleigh, float mieG, float fogRange, float fogDensity, float distance)
{
    vec3 sunUp = vec3(0, 1, 0);
    vec3 sunDirection = normalize(L);

    float zenithAngle = saturate(dot(V, sunUp));

    vec3 inscatteringMie = waveLambdaMie;
    vec3 inscatteringRayleigh = waveLambdaRayleigh;

    float cosTheta = dot(V, sunDirection);

    vec3 betaMie = inscatteringMie * ComputePhaseMie(cosTheta, mieG);
    vec3 betaRayleigh = inscatteringRayleigh * ComputePhaseRayleigh(cosTheta);

    vec3 inscattering = betaMie / (inscatteringMie + inscatteringRayleigh);

    vec3 extinction = exp(-(inscatteringMie + inscatteringRayleigh) * distance * fogDensity);
    extinction = pow(extinction, fogRange);

    return max(0, inscattering * (1.0 - extinction));
}

vec3 ComputeSkyFogRayleigh(vec3 V, vec3 L, vec3 waveLambdaMie, vec3 waveLambdaRayleigh, float mieG, float fogRange, float fogDensity, float distance)
{
    vec3 sunUp = vec3(0, 1, 0);
    vec3 sunDirection = normalize(L);

    float zenithAngle = saturate(dot(V, sunUp));

    vec3 inscatteringMie = waveLambdaMie;
    vec3 inscatteringRayleigh = waveLambdaRayleigh;

    float cosTheta = dot(V, sunDirection);

    vec3 betaMie = inscatteringMie * ComputePhaseMie(cosTheta, mieG);
    vec3 betaRayleigh = inscatteringRayleigh * ComputePhaseRayleigh(cosTheta);

    vec3 inscattering = betaRayleigh / (inscatteringMie + inscatteringRayleigh);

    vec3 extinction = exp(-(inscatteringMie + inscatteringRayleigh) * distance * fogDensity);
    extinction = pow(extinction, fogRange);

    return max(0, inscattering * (1.0 - extinction));
}

vec3 ComputeSkyFogApproximation(vec3 V, vec3 L, vec3 waveLambdaMie, vec3 waveLambdaRayleigh, float mieG, float fogRange, float fogDensity, float distance)
{
    vec3 sunUp = vec3(0, 1, 0);
    vec3 sunDirection = normalize(L);

    float zenithAngle = saturate(dot(V, sunUp));

    vec3 inscatteringMie = waveLambdaMie;
    vec3 inscatteringRayleigh = waveLambdaRayleigh;

    float cosTheta = dot(V, sunDirection);

    vec3 betaMie = inscatteringMie * ComputePhaseMie(cosTheta, mieG);
    vec3 betaRayleigh = inscatteringRayleigh * ComputePhaseRayleigh(cosTheta);
    vec3 betaScattering = (betaMie + betaRayleigh);

    vec3 inscattering = betaScattering / (inscatteringMie + inscatteringRayleigh);

    vec3 extinction = exp(-(inscatteringMie + inscatteringRayleigh) * distance * fogDensity);
    extinction = pow(extinction, fogRange);

    return max(0, inscattering * (1.0 - extinction));
}