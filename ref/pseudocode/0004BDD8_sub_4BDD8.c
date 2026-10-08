/* ea:   0x0004BDD8 */
/* name: sub_4BDD8 */
/* size: 396 */
int __fastcall sub_4BDD8(int a1, int a2)
{
  int result; // r0
  int v5; // r6
  _DWORD *i; // r4
  unsigned __int8 *v7; // r4
  int v8; // t1
  int v9; // r0
  int v10; // r0
  int v11; // [sp+Ch] [bp-40h] BYREF
  __int16 v12; // [sp+10h] [bp-3Ch]
  unsigned __int8 v13; // [sp+12h] [bp-3Ah]
  int v14[12]; // [sp+1Ch] [bp-30h] BYREF

  result = *(_DWORD *)(a1 + 20);
  if ( result != 0 )
  {
    sub_4A338(v14);
    sub_4DEAC(a2 + 16, *(_DWORD *)(a2 + 16), *(_DWORD *)(a2 + 20));
    sub_4DEAC(a2 + 4, *(_DWORD *)(a2 + 4), *(_DWORD *)(a2 + 8));
    v5 = *(_DWORD *)(a1 + 24) + 608;
    for ( i = *(_DWORD **)(*(_DWORD *)(a1 + 152) + 8); i != *(_DWORD **)(a1 + 152); i = sub_16A30(i) )
    {
      v13 = *((_BYTE *)i + 16);
      v12 = *((_WORD *)i + 9);
      v11 = sub_53BC4(v5, v13, *(unsigned __int8 *)(*(_DWORD *)(a1 + 20) + 48), 1);
      sub_4DCD8(a2 + 4, &v11);
    }
    sub_539E4(v5, *(unsigned __int8 *)(*(_DWORD *)(a1 + 20) + 48), *(unsigned __int8 *)(*(_DWORD *)(a1 + 20) + 49), v14);
    v7 = (unsigned __int8 *)v14[0];
    while ( v7 != (unsigned __int8 *)v14[1] )
    {
      v8 = *v7;
      v7 += 36;
      v9 = *(_DWORD *)(a1 + 20);
      v13 = v8;
      v10 = sub_53A4C(v5, a1, v8, *(unsigned __int8 *)(v9 + 48), 1);
      v12 = 0;
      v11 = v10;
      sub_4DCD8(a2 + 16, &v11);
    }
    *(_DWORD *)a2 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
    *(_WORD *)(a2 + 28) = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    return sub_2F710(v14);
  }
  else
  {
    *(_DWORD *)a2 = 0;
    *(_DWORD *)(a2 + 4) = 0;
    *(_DWORD *)(a2 + 8) = 0;
    *(_DWORD *)(a2 + 12) = 0;
    *(_DWORD *)(a2 + 16) = 0;
    *(_DWORD *)(a2 + 20) = 0;
    *(_DWORD *)(a2 + 24) = 0;
    *(_DWORD *)(a2 + 28) = 0;
  }
  return result;
}

