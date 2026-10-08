/* ea:   0x00016A30 */
/* name: sub_16A30 */
/* size: 112 */
_DWORD *__fastcall sub_16A30(_DWORD *a1)
{
  _DWORD *v1; // r2
  _DWORD *v2; // r1
  int v3; // r0
  int v4; // r1
  _DWORD *result; // r0

  v1 = a1;
  v2 = (_DWORD *)a1[3];
  if ( v2 != nullptr )
  {
    v3 = v2[2];
    if ( v3 != 0 )
    {
      do
      {
        v4 = v3;
        v3 = *(_DWORD *)(v3 + 8);
      }
      while ( v3 != 0 );
      return (_DWORD *)v4;
    }
    return v2;
  }
  v2 = (_DWORD *)a1[1];
  if ( a1 == (_DWORD *)v2[3] )
  {
    do
    {
      v1 = v2;
      v2 = (_DWORD *)v2[1];
    }
    while ( v1 == (_DWORD *)v2[3] );
  }
  result = v1;
  if ( (_DWORD *)v1[3] != v2 )
    return v2;
  return result;
}

