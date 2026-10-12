/* ea:   0x00039948 */
/* name: sub_39948 */
/* size: 136 */
_DWORD *__fastcall sub_39948(int a1, int *a2)
{
  char v3; // r4
  _DWORD *v4; // r6
  _DWORD *v5; // r2
  int v6; // r1

  v3 = 1;
  v4 = *(_DWORD **)a1;
  v5 = *(_DWORD **)(*(_DWORD *)a1 + 4);
  if ( v5 != nullptr )
  {
    v6 = *a2;
    do
    {
      if ( v5[4] < v6 )
      {
        v5 = (_DWORD *)v5[3];
      }
      else
      {
        v4 = v5;
        v5 = (_DWORD *)v5[2];
      }
    }
    while ( v5 != nullptr );
  }
  if ( v4 == *(_DWORD **)a1 )
    return *(_DWORD **)a1;
  if ( *a2 >= v4[4] )
    v3 = 0;
  if ( v3 != 0 )
    return *(_DWORD **)a1;
  else
    return v4;
}

