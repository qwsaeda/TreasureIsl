/* ea:   0x00035190 */
/* name: sub_35190 */
/* size: 212 */
_DWORD *__fastcall sub_35190(_DWORD *result, _DWORD *a2)
{
  _DWORD *v2; // r5
  int v4; // r1
  int i; // r0
  int v6; // r1
  int j; // r0

  v2 = result;
  if ( result != a2 )
  {
    sub_16ED8(result);
    v2[1] = 0;
    *((_BYTE *)v2 + 8) = *((_BYTE *)a2 + 8);
    if ( *(_DWORD *)(*a2 + 4) != 0 )
    {
      *(_DWORD *)(*v2 + 4) = sub_38898(v2, *(_DWORD *)(*a2 + 4), *v2);
      v4 = *(_DWORD *)(*v2 + 4);
      for ( i = *(_DWORD *)(v4 + 8); i != 0; i = *(_DWORD *)(i + 8) )
        v4 = i;
      *(_DWORD *)(*v2 + 8) = v4;
      v6 = *(_DWORD *)(*v2 + 4);
      for ( j = *(_DWORD *)(v6 + 12); j != 0; j = *(_DWORD *)(j + 12) )
        v6 = j;
      *(_DWORD *)(*v2 + 12) = v6;
      v2[1] = a2[1];
      return v2;
    }
    else
    {
      *(_DWORD *)(*v2 + 4) = 0;
      *(_DWORD *)(*v2 + 8) = *v2;
      *(_DWORD *)(*v2 + 12) = *v2;
      return v2;
    }
  }
  return result;
}

