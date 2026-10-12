/* ea:   0x000645B4 */
/* name: sub_645B4 */
/* size: 424 */
void *__fastcall sub_645B4(int a1, int a2, int a3)
{
  void *v6; // r0
  void *v7; // r5
  int v8; // r1
  int (*v9)(); // r2
  int v11; // r4
  _DWORD v12[8]; // [sp+0h] [bp-20h] BYREF

  if ( a2 == 0 )
    __debugbreak();
  if ( (unsigned __int8)sub_64D94(a1, a2, a3) == 0
    || (unsigned __int8)sub_64DC0(a1, a2, a3, v12) == 0
    || *(_BYTE *)(a1 + 4) != 0 && (unsigned __int8)sub_64E70(a1, v12[0], *(_DWORD *)(a1 + 8)) == 0 )
  {
    return nullptr;
  }
  switch ( v12[0] )
  {
    case 4:
      v6 = (void *)sub_64954(a1, a2, a3, v12);
      break;
    case 8:
      v6 = (void *)sub_6475C(a1, a2, a3, v12);
      break;
    case 0x18:
      v6 = (void *)sub_64B80(a1, a2, a3, v12);
      break;
    default:
      return nullptr;
  }
  v7 = v6;
  if ( v6 == nullptr )
    return nullptr;
  if ( *(_BYTE *)(a1 + 4) == 0 )
    return v7;
  v8 = *(_DWORD *)(a1 + 8);
  if ( v12[0] == v8 )
    return v7;
  if ( v12[0] == 8 )
  {
    if ( v8 == 16 )
    {
      v9 = sub_6A4F8;
      goto LABEL_27;
    }
    if ( v8 == 24 )
    {
      v9 = sub_6A58C;
      goto LABEL_27;
    }
LABEL_28:
    free(v6);
    return nullptr;
  }
  if ( v12[0] == 16 )
  {
    if ( v8 == 24 )
    {
      v9 = sub_6A6E4;
      goto LABEL_27;
    }
    goto LABEL_28;
  }
  if ( v12[0] != 24 || v8 != 16 )
    goto LABEL_28;
  v9 = (int (*)())sub_6A7B4;
LABEL_27:
  if ( v9 == nullptr )
    goto LABEL_28;
  v11 = ((int (__fastcall *)(void *))v9)(v6);
  v6 = v7;
  if ( v11 == 0 )
    goto LABEL_28;
  free(v7);
  return (void *)v11;
}

