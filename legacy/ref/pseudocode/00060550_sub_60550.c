/* ea:   0x00060550 */
/* name: sub_60550 */
/* size: 84 */
_DWORD *__fastcall sub_60550(int a1, unsigned int *a2)
{
  _DWORD *v2; // r3
  _DWORD *v3; // r2
  unsigned int v4; // r1

  v2 = *(_DWORD **)a1;
  v3 = *(_DWORD **)(*(_DWORD *)a1 + 4);
  if ( v3 != nullptr )
  {
    v4 = *a2;
    do
    {
      if ( v3[4] < v4 )
      {
        v3 = (_DWORD *)v3[3];
      }
      else
      {
        v2 = v3;
        v3 = (_DWORD *)v3[2];
      }
    }
    while ( v3 != nullptr );
  }
  return v2;
}

