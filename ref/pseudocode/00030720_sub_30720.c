/* ea:   0x00030720 */
/* name: sub_30720 */
/* size: 240 */
BOOL __fastcall sub_30720(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r6
  _DWORD *v6; // r4
  _WORD *v7; // r3
  unsigned int v8; // r2

  v4 = a2;
  if ( sub_16D1C((int)a1) > a2 )
  {
    v6 = *(_DWORD **)(*a1 + 8);
    while ( v4 != 0 )
    {
      if ( *((_WORD *)v6 + 9) != 0 )
      {
        sub_30810(a3, *((unsigned __int8 *)v6 + 16), 1);
        v7 = (_WORD *)v6 + 9;
        v8 = *((unsigned __int16 *)v6 + 9);
        if ( v8 > 1 )
          *v7 = v8 - 1;
        else
          *v7 = 0;
      }
      v6 = sub_16A30(v6);
      --v4;
      if ( v6 == (_DWORD *)*a1 )
        v6 = *(_DWORD **)(*a1 + 8);
    }
    return sub_16F5C((int)a1);
  }
  else
  {
    sub_35190(a3, a1);
    sub_16ED8(a1);
    sub_16F5C((int)a1);
    return sub_16F5C((int)a1);
  }
}

