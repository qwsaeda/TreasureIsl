/* ea:   0x0005ADA4 */
/* name: sub_5ADA4 */
/* size: 92 */
unsigned int __fastcall sub_5ADA4(_DWORD *a1, unsigned int a2)
{
  unsigned int result; // r0
  unsigned int v4; // r5
  int v5; // r4

  result = a1[21];
  if ( a2 > 0x40 )
    a2 = 64;
  a1[32] = a2;
  v4 = 0;
  if ( result != 0 )
  {
    v5 = 0;
    do
    {
      (**(void (__fastcall ***)(int, _DWORD))(v5 + a1[22]))(v5 + a1[22], *(_DWORD *)(v5 + a1[22] + 8));
      result = a1[21];
      ++v4;
      v5 += 60;
    }
    while ( v4 < result );
  }
  return result;
}

