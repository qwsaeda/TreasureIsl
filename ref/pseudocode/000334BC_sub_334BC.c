/* ea:   0x000334BC */
/* name: sub_334BC */
/* size: 140 */
int __fastcall sub_334BC(int a1)
{
  void *v3[4]; // [sp+0h] [bp-28h] BYREF
  void *v4[6]; // [sp+10h] [bp-18h] BYREF

  sub_1333C(v4);
  sub_1333C(v3);
  sub_332DC((_DWORD *)(a1 + 4));
  sub_1309C((int)v3);
  sub_162DC(v3, (char *)aDataGlpicsPak);
  if ( (unsigned __int8)sub_33548(a1, v3[0], a1 + 4) != 0 )
  {
    sub_132A0(v3);
    sub_12F48(v4);
    return 1;
  }
  else
  {
    sub_12EA0((int)v3);
    sub_12EA0((int)v4);
    return 0;
  }
}

