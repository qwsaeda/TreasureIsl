/* ea:   0x0003028C */
/* name: sub_3028C */
/* size: 240 */
_DWORD *__fastcall sub_3028C(_DWORD *a1, int a2)
{
  bool v4; // zf
  int v6; // r1
  int i; // r0
  int v8; // r1
  int j; // r0
  char v10; // [sp+0h] [bp-10h] BYREF

  sub_329D8(a1, &v10);
  a1[1] = 0;
  *((_BYTE *)a1 + 8) = *(_BYTE *)(a2 + 8);
  v4 = *(_DWORD *)(*(_DWORD *)a2 + 4) == 0;
  *(_BYTE *)*a1 = 0;
  if ( v4 )
  {
    *(_DWORD *)(*a1 + 4) = 0;
    *(_DWORD *)(*a1 + 8) = *a1;
    *(_DWORD *)(*a1 + 12) = *a1;
    a1[1] = *(_DWORD *)(a2 + 4);
    return a1;
  }
  else
  {
    *(_DWORD *)(*a1 + 4) = sub_36284(a1, *(_DWORD *)(*(_DWORD *)a2 + 4), *a1);
    v6 = *(_DWORD *)(*a1 + 4);
    for ( i = *(_DWORD *)(v6 + 8); i != 0; i = *(_DWORD *)(i + 8) )
      v6 = i;
    *(_DWORD *)(*a1 + 8) = v6;
    v8 = *(_DWORD *)(*a1 + 4);
    for ( j = *(_DWORD *)(v8 + 12); j != 0; j = *(_DWORD *)(j + 12) )
      v8 = j;
    *(_DWORD *)(*a1 + 12) = v8;
    a1[1] = *(_DWORD *)(a2 + 4);
    return a1;
  }
}

