/* ea:   0x00058FFC */
/* name: sub_58FFC */
/* size: 100 */
int __fastcall sub_58FFC(unsigned int a1)
{
  return _byteswap_ushort(a1) | ((HIBYTE(a1) | (BYTE2(a1) << 8)) << 16);
}

