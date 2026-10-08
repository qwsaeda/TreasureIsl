/* ea:   0x00039ADC */
/* name: sub_39ADC */
/* size: 52 */
_BYTE *__fastcall sub_39ADC(int a1, _BYTE *a2)
{
  _BYTE *v3; // r3

  v3 = sub_344C4(a1, 1);
  if ( v3 != (_BYTE *)-16 )
  {
    v3[16] = *a2;
    v3[17] = a2[1];
  }
  return v3;
}

