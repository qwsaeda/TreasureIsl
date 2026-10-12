/* ea:   0x0003565C */
/* name: sub_3565C */
/* size: 320 */
int __fastcall sub_3565C(char **a1, int a2, unsigned __int8 *a3)
{
  char *v6; // r2
  char *v7; // r5
  int v8; // r4
  bool v9; // r0
  unsigned int v10; // r1
  int v11; // r3
  int v12; // r1
  int v14; // r1
  _BYTE v15[4]; // [sp+8h] [bp-30h] BYREF
  _BYTE v16[4]; // [sp+Ch] [bp-2Ch] BYREF
  int v17; // [sp+10h] [bp-28h] BYREF
  char v18; // [sp+14h] [bp-24h]

  v6 = *a1;
  v7 = *a1;
  v8 = *((_DWORD *)*a1 + 1);
  v9 = true;
  while ( v8 != 0 )
  {
    v7 = (char *)v8;
    v10 = *a3;
    v9 = v10 < *(unsigned __int8 *)(v8 + 16);
    if ( v10 >= *(unsigned __int8 *)(v8 + 16) )
      v8 = *(_DWORD *)(v8 + 12);
    else
      v8 = *(_DWORD *)(v8 + 8);
  }
  v11 = (int)v7;
  if ( v9 )
  {
    if ( v7 == *((char **)v6 + 2) )
    {
      v12 = *(_DWORD *)sub_38778(a1, v15, v8, v7, a3, 0);
      *(_BYTE *)(a2 + 4) = 1;
      *(_DWORD *)a2 = v12;
      return a2;
    }
    v11 = sub_17924(v7);
  }
  if ( *(unsigned __int8 *)(v11 + 16) >= (unsigned int)*a3 )
  {
    v17 = v11;
    v18 = 0;
    sub_3579C(a2, &v17);
  }
  else
  {
    v14 = *(_DWORD *)sub_38778(a1, v16, v8, v7, a3, 0);
    *(_BYTE *)(a2 + 4) = 1;
    *(_DWORD *)a2 = v14;
  }
  return a2;
}

