/* ea:   0x00039364 */
/* name: sub_39364 */
/* size: 232 */
int *__fastcall sub_39364(_DWORD *a1, int *a2, int a3, _DWORD *a4, _DWORD *a5, int a6)
{
  int v9; // r4
  _DWORD *v10; // r0
  int *v11; // r1
  int v12; // r0
  int v13; // r1
  _DWORD *v14; // t1

  if ( a4 == (_DWORD *)*a1 || a6 == 0 && (a3 != 0 || *a5 < a4[4]) )
  {
    v12 = sub_39C00(a1, a5);
    v9 = v12;
    a4[2] = v12;
    v13 = *a1;
    if ( a4 == (_DWORD *)*a1 )
    {
      *(_DWORD *)(v13 + 4) = v12;
      *(_DWORD *)(*a1 + 12) = v12;
      goto LABEL_11;
    }
    v14 = *(_DWORD **)(v13 + 8);
    v11 = (int *)(v13 + 8);
    v10 = v14;
  }
  else
  {
    v9 = sub_39C00(a1, a5);
    a4[3] = v9;
    v11 = (int *)(*a1 + 12);
    v10 = (_DWORD *)*v11;
  }
  if ( a4 == v10 )
    *v11 = v9;
LABEL_11:
  *(_DWORD *)(v9 + 4) = a4;
  *(_DWORD *)(v9 + 8) = 0;
  *(_DWORD *)(v9 + 12) = 0;
  sub_183FC((_BYTE *)v9, (_DWORD *)(*a1 + 4));
  ++a1[1];
  *a2 = v9;
  return a2;
}

