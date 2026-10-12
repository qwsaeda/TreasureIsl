/* ea:   0x00038974 */
/* name: sub_38974 */
/* size: 264 */
_DWORD *__fastcall sub_38974(_DWORD *a1, _DWORD *a2, int a3, _DWORD *a4, _DWORD *a5, int a6)
{
  _DWORD *v9; // r0
  _DWORD *v10; // r5
  _DWORD *v11; // r0
  _DWORD *v12; // r1
  int v13; // r0
  int v14; // r1
  _DWORD *v15; // t1

  if ( a4 == (_DWORD *)*a1 || a6 == 0 && (a3 != 0 || *a5 < a4[4]) )
  {
    v13 = sub_39AA0(a1, a5);
    v10 = (_DWORD *)v13;
    a4[2] = v13;
    v14 = *a1;
    if ( a4 == (_DWORD *)*a1 )
    {
      *(_DWORD *)(v14 + 4) = v13;
      *(_DWORD *)(*a1 + 12) = v13;
      goto LABEL_13;
    }
    v15 = *(_DWORD **)(v14 + 8);
    v12 = (_DWORD *)(v14 + 8);
    v11 = v15;
  }
  else
  {
    v9 = sub_34438((int)a1, 1);
    v10 = v9;
    if ( v9 != (_DWORD *)-16 )
    {
      v9[4] = *a5;
      memcpy(v9 + 5, a5 + 1, 6u);
    }
    a4[3] = v10;
    v12 = (_DWORD *)(*a1 + 12);
    v11 = (_DWORD *)*v12;
  }
  if ( a4 == v11 )
    *v12 = v10;
LABEL_13:
  v10[1] = a4;
  v10[2] = 0;
  v10[3] = 0;
  sub_183FC(v10, (_DWORD *)(*a1 + 4));
  ++a1[1];
  *a2 = v10;
  return a2;
}

