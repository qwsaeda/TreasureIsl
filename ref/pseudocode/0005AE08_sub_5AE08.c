/* ea:   0x0005AE08 */
/* name: sub_5AE08 */
/* size: 96 */
int __fastcall sub_5AE08(_DWORD *a1, unsigned int a2)
{
  int result; // r0
  unsigned int v4; // r5
  int v5; // r4

  result = a1[19];
  if ( a2 > 0x40 )
    a2 = 64;
  a1[31] = a2;
  v4 = 0;
  if ( result != 0 )
  {
    v5 = 0;
    do
    {
      (**(void (__fastcall ***)(int, _DWORD))(v5 + a1[20]))(v5 + a1[20], *(_DWORD *)(v5 + a1[20] + 8));
      ++v4;
      result = v5 + 1280;
      v5 += 1304;
    }
    while ( v4 < a1[19] );
  }
  return result;
}

