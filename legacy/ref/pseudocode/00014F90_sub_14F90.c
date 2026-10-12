/* ea:   0x00014F90 */
/* name: sub_14F90 */
/* size: 620 */
int sub_14F90(int a1, int a2, ...)
{
  int v4; // r9
  int v5; // r4
  unsigned __int8 v6; // r5
  _DWORD *v7; // r6
  int v8; // t1
  int v9; // r4
  _DWORD *v10; // r0
  _DWORD *v11; // r0
  int result; // r0
  int v13; // r0
  int v14; // r0
  _DWORD *v15; // r0
  int varg_r2; // [sp+34h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+34h] [bp-8h]
  int varg_r3; // [sp+38h] [bp-4h]
  va_list va1; // [sp+3Ch] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v4 = varg_r3;
  sub_16B48(a2 + 164, 12);
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  v5 = 0;
  switch ( *(_BYTE *)(*(_DWORD *)sub_1623C(varg_r3 + 508, (int *)varg_r2a) + 49) )
  {
    case 1:
      v5 = 3;
      break;
    case 2:
      v5 = 5;
      break;
    case 3:
      v5 = 7;
      break;
    case 4:
      v5 = 8;
      break;
    default:
      break;
  }
  v6 = rand() % v5 + 1;
  v7 = *(_DWORD **)(*(_DWORD *)sub_1623C(v4 + 508, (int *)varg_r2a) + 28);
  if ( *v7 != 0 )
  {
    do
    {
      v8 = v7[1];
      ++v7;
    }
    while ( v8 != 0 );
  }
  v9 = v6;
  if ( v6 != 0 )
  {
    do
    {
      v10 = (_DWORD *)sub_1623C(v4 + 508, (int *)varg_r2a);
      if ( (unsigned __int8)sub_17A90(*v10, v9) != 0 )
        break;
      v6 = v9 - 1;
      v9 = (unsigned __int8)(v9 - 1);
    }
    while ( v9 != 0 );
  }
  if ( v7 == *(_DWORD **)(*(_DWORD *)sub_1623C(v4 + 508, (int *)varg_r2a) + 32) || v6 == 0 )
  {
    if ( *(_DWORD *)(a2 + 8) == 1 )
    {
      v13 = sub_12F04(aO);
      sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v13]);
      return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    }
    else
    {
      v14 = sub_12F04(aU_0);
      sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v14]);
      sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
      v15 = (_DWORD *)sub_186F0(v4);
      return sub_3AA9C(&dword_81184, aU, *v15);
    }
  }
  else
  {
    sub_3AA9C(&dword_81184, aB, *(&off_7EDD8 + v6 - 1));
    if ( *(_DWORD *)(a2 + 8) == 1 )
    {
      result = sub_3AA9C(&dword_81184, aU, &unk_7D724);
    }
    else
    {
      v11 = (_DWORD *)sub_186F0(v4);
      result = sub_3AA9C(&dword_81184, aU, *v11);
    }
    *v7 = v6;
  }
  return result;
}

