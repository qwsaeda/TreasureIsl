/* ea:   0x0005F498 */
/* name: sub_5F498 */
/* size: 196 */
int __fastcall sub_5F498(int a1, int a2, int a3, int *a4)
{
  int v7; // r6
  int v8; // r4
  int v10; // [sp+0h] [bp-3Ch] BYREF
  void *v11[3]; // [sp+4h] [bp-38h] BYREF
  int v12; // [sp+10h] [bp-2Ch] BYREF
  void *v13[10]; // [sp+14h] [bp-28h] BYREF

  v10 = a2;
  sub_1333C(v11);
  v7 = sub_61918(a1 + 4, &v10);
  sub_12F48(v11);
  if ( v7 == *(_DWORD *)(a1 + 4) )
    return 0;
  v12 = a3;
  sub_1333C(v13);
  v8 = sub_61890(v7 + 32, &v12);
  sub_132A0(v13);
  if ( v8 == *(_DWORD *)(v7 + 32) )
    return 0;
  *a4 = wtol(*(const wchar_t **)(v8 + 32));
  return 1;
}

